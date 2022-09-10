package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"path"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"
	"time"

	firecracker "github.com/firecracker-microvm/firecracker-go-sdk"
	"github.com/firecracker-microvm/firecracker-go-sdk/client/models"
	"github.com/imroc/req"
	"github.com/rs/xid"
	log "github.com/sirupsen/logrus"
	"github.com/streadway/amqp"
)

type jobQueue struct {
	ch    *amqp.Channel
	conn  *amqp.Connection
	jobsQ amqp.Queue
	jobs  <-chan amqp.Delivery
}

type agentRunReq struct {
	ID       string `json:"id"`
	Language string `json:"language"`
	Code     string `json:"code"`
	Variant  string `json:"variant"`
}

type agentExecRes struct {
	Message      string `json:"message"`
	Error        string `json:"error"`
	StdErr       string `json:"stderr"`
	StdOut       string `json:"stdout"`
	ExecDuration int    `json:"exec_duration"`
	MemUsage     int64  `json:"mem_usage"`
}

type jobStatus struct {
	ID           string `json:"id"`
	Status       string `json:"status"`
	Message      string `json:"message"`
	Error        string `json:"error"`
	StdErr       string `json:"stderr"`
	StdOut       string `json:"stdout"`
	ExecDuration int    `json:"exec_duration"`
	MemUsage     int64  `json:"mem_usage"`
}

var queue jobQueue

type benchJob struct {
	ID       string `json:"id"`
	Language string `json:"language"`
	Code     string `json:"code"`
}

func (q jobQueue) setjobStatus(ctx context.Context, job benchJob, status string, res agentExecRes) error {
	log.WithField("status", status).Info("Set job status")
	jobStatus := &jobStatus{
		ID:      job.ID,
		Status:  status,
		Message: res.Message,
		Error:   res.Error,
		StdErr:  "",
		StdOut:  "",
	}
	b, err := json.Marshal(jobStatus)
	if err != nil {
		return err
	}
	err = q.ch.Publish(
		"jobs_status_ex", // exchange
		"jobs_status_rk", // routing key
		false,            // mandatory
		false,            // immediate
		amqp.Publishing{
			ContentType: "text/plain",
			Body:        b,
		})
	return err
}

func (q jobQueue) setjobReceived(ctx context.Context, job benchJob) error {
	return q.setjobStatus(ctx, job, "received", agentExecRes{})
}

func (q jobQueue) setjobRunning(ctx context.Context, job benchJob) error {
	return q.setjobStatus(ctx, job, "running", agentExecRes{})
}

func (q jobQueue) setjobFailed(ctx context.Context, job benchJob, res agentExecRes) error {
	return q.setjobStatus(ctx, job, "failed", res)
}
func (q jobQueue) setjobResult(ctx context.Context, job benchJob, res agentExecRes) error {
	jobStatus := &jobStatus{
		ID:           job.ID,
		Status:       "done",
		Message:      res.Message,
		Error:        res.Error,
		StdErr:       res.StdErr,
		StdOut:       res.StdOut,
		ExecDuration: res.ExecDuration,
		MemUsage:     res.MemUsage,
	}
	log.WithField("jobStatus", jobStatus).Info("Set job result")

	b, err := json.Marshal(jobStatus)
	if err != nil {
		return err
	}
	err = q.ch.Publish(
		"jobs_status_ex", // exchange
		"jobs_status_rk", // routing key
		false,            // mandatory
		false,            // immediate
		amqp.Publishing{
			ContentType: "text/plain",
			Body:        b,
		})
	return err
}

func copy(src string, dst string) error {
	data, err := ioutil.ReadFile(src)
	if err != nil {
		return err
	}
	err = ioutil.WriteFile(dst, data, 0644)
	return err
}

func newJobQueue(endpoint string) jobQueue {
	conn, err := amqp.Dial(endpoint)
	if err != nil {
		log.WithError(err).Fatal("Failed to connect to RabbitMQ")
	}

	ch, err := conn.Channel()
	if err != nil {
		log.WithError(err).Fatal("Failed to open a channel")
	}

	err = ch.ExchangeDeclare(
		"jobs_ex", // name
		"direct",  // type
		true,      // durable
		false,     // auto-deleted
		false,     // internal
		false,     // no-wait
		nil,       // arguments
	)
	if err != nil {
		log.WithError(err).Fatal("Failed to declare an exchange")
	}

	jobsQ, err := ch.QueueDeclare(
		"jobs_q", // name
		true,     // durable
		false,    // delete when unused
		false,    // exclusive
		false,    // no-wait
		nil,      // arguments
	)
	if err != nil {
		log.WithError(err).Fatal("Failed to declare a queue")
	}

	err = ch.QueueBind(
		jobsQ.Name, // queue name
		"jobs_rk",  // routing key
		"jobs_ex",  // exchange
		false,
		nil)
	if err != nil {
		log.WithError(err).Fatal("Failed to bind a queue")
	}
	jobs, err := ch.Consume(
		jobsQ.Name, // queue
		"",         // consumer
		true,       // auto-ack
		false,      // exclusive
		false,      // no-local
		false,      // no-wait
		nil,        // args
	)
	if err != nil {
		log.WithError(err).Fatal("Failed to register a consumer")
	}

	return jobQueue{
		ch,
		conn,
		jobsQ,
		jobs,
	}
}

func (q jobQueue) getQueueForJob(ctx context.Context) error {
	return q.ch.ExchangeDeclare(
		"jobs_status_ex", // name
		"direct",         // type
		false,            // durable
		false,            // auto-deleted
		false,            // internal
		false,            // no-wait
		nil,              // arguments
	)
}

func deleteVMMSockets() {
	log.Debug("cc")
	dir, err := ioutil.ReadDir(os.TempDir())
	if err != nil {
		log.WithError(err).Error("Failed to read directory")
	}
	for _, d := range dir {
		log.WithField("d", d.Name()).Debug("considering")
		if strings.Contains(d.Name(), fmt.Sprintf(".firecracker.sock-%d-", os.Getpid())) {
			log.WithField("d", d.Name()).Debug("should delete")
			os.Remove(path.Join([]string{"tmp", d.Name()}...))
		}
	}
}

type runningFirecracker struct {
	vmmCtx    context.Context
	vmmCancel context.CancelFunc
	vmmID     string
	machine   *firecracker.Machine
	ip        net.IP
}

func main() {

	defer deleteVMMSockets()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	WarmVMs := make(chan runningFirecracker, 10)
	go fillVMPool(ctx, WarmVMs)
	installSignalHandlers()

	log.SetReportCaller(true)

	queue = newJobQueue("amqp://admin:password@localhost:5672//dev")
	defer queue.ch.Close()
	defer queue.conn.Close()

	err := queue.getQueueForJob(ctx)
	if err != nil {
		log.WithError(err).Fatal("Failed to get status queue")
		return
	}

	log.Info("Waiting for RabbitMQ jobs...")

	for d := range queue.jobs {
		log.Printf("Received a message: %s", d.Body)

		var job benchJob
		err := json.Unmarshal([]byte(d.Body), &job)
		if err != nil {
			log.WithError(err).Error("Received invalid job")
			continue
		}

		go job.run(ctx, WarmVMs)
	}

}

func waitForVMToBoot(ctx context.Context, ip net.IP) error {
	// Query the agent until it provides a valid response
	req.SetTimeout(500 * time.Millisecond)
	for {
		select {
		case <-ctx.Done():
			// Timeout
			return ctx.Err()
		default:
			res, err := req.Get("http://" + ip.String() + ":8080/health")
			if err != nil {
				log.WithError(err).Info("VM not ready yet")
				time.Sleep(time.Second)
				continue
			}

			if res.Response().StatusCode != 200 {
				time.Sleep(time.Second)
				log.Info("VM not ready yet")
			} else {
				log.WithField("ip", ip).Info("VM agent ready")
				return nil
			}
			time.Sleep(time.Second)
		}

	}
}

func fillVMPool(ctx context.Context, WarmVMs chan<- runningFirecracker) {
	for {
		select {
		case <-ctx.Done():
			// Program is stopping, WarmVMs will be cleaned up, bye
			return
		default:
			vm, err := createAndStartVM(ctx)
			if err != nil {
				log.Error("failed to create VMM")
				time.Sleep(time.Second)
				continue
			}

			log.WithField("ip", vm.ip).Info("New VM created and started")

			// Don't wait forever, if the VM is not available after 10s, move on
			ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
			defer cancel()

			err = waitForVMToBoot(ctx, vm.ip)
			if err != nil {
				log.WithError(err).Info("VM not ready yet")
				vm.vmmCancel()
				continue
			}

			// Add the new microVM to the pool.
			// If the pool is full, this line will block until a slot is available.
			WarmVMs <- *vm
		}
	}
}

func getSocketPath(vmmID string) string {
	filename := strings.Join([]string{
		".firecracker.sock",
		strconv.Itoa(os.Getpid()),
		vmmID,
	},
		"-",
	)
	dir := os.TempDir()

	return filepath.Join(dir, filename)
}

func getFirecrackerConfig(vmmID string) (firecracker.Config, error) {
	socket := getSocketPath(vmmID)
	return firecracker.Config{
		SocketPath:      socket,
		KernelImagePath: "../agent/kernel",
		LogPath:         fmt.Sprintf("%s.log", socket),
		KernelArgs:      "console=ttyS0 reboot=k panic=1 pci=off",
		Drives: []models.Drive{{
			DriveID:      firecracker.String("1"),
			PathOnHost:   firecracker.String("/tmp/rootfs-" + vmmID + ".ext4"),
			IsRootDevice: firecracker.Bool(true),
			IsReadOnly:   firecracker.Bool(false),
			RateLimiter: firecracker.NewRateLimiter(
				// bytes/s
				models.TokenBucket{
					OneTimeBurst: firecracker.Int64(1024 * 1024), // 1 MiB/s
					RefillTime:   firecracker.Int64(500),         // 0.5s
					Size:         firecracker.Int64(1024 * 1024),
				},
				// ops/s
				models.TokenBucket{
					OneTimeBurst: firecracker.Int64(100),  // 100 iops
					RefillTime:   firecracker.Int64(1000), // 1s
					Size:         firecracker.Int64(100),
				}),
		}},
		NetworkInterfaces: []firecracker.NetworkInterface{{
			// Use CNI to get dynamic IP
			CNIConfiguration: &firecracker.CNIConfiguration{
				NetworkName: "fcnet",
				IfName:      "veth0",
			},
		}},
		MachineCfg: models.MachineConfiguration{
			VcpuCount:  firecracker.Int64(1),
			MemSizeMib: firecracker.Int64(256),
		},
	}, nil
}

func createAndStartVM(ctx context.Context) (*runningFirecracker, error) {
	vmmID := xid.New().String()

	copy("../agent/rootfs.ext4", "/tmp/rootfs-"+vmmID+".ext4")

	fcCfg, err := getFirecrackerConfig(vmmID)
	if err != nil {
		log.Errorf("Error: %s", err)
		return nil, err
	}
	logger := log.New()

	if false { // TODO
		log.SetLevel(log.DebugLevel)
		logger.SetLevel(log.DebugLevel)
	}

	machineOpts := []firecracker.Opt{
		firecracker.WithLogger(log.NewEntry(logger)),
	}

	firecrackerBinary, err := exec.LookPath("firecracker")
	if err != nil {
		return nil, err
	}

	finfo, err := os.Stat(firecrackerBinary)
	if os.IsNotExist(err) {
		return nil, fmt.Errorf("binary %q does not exist: %v", firecrackerBinary, err)
	}
	if err != nil {
		return nil, fmt.Errorf("failed to stat binary, %q: %v", firecrackerBinary, err)
	}

	if finfo.IsDir() {
		return nil, fmt.Errorf("binary, %q, is a directory", firecrackerBinary)
	} else if finfo.Mode()&0111 == 0 {
		return nil, fmt.Errorf("binary, %q, is not executable. Check permissions of binary", firecrackerBinary)
	}

	// if the jailer is used, the final command will be built in NewMachine()
	if fcCfg.JailerCfg == nil {
		cmd := firecracker.VMCommandBuilder{}.
			WithBin(firecrackerBinary).
			WithSocketPath(fcCfg.SocketPath).
			// WithStdin(os.Stdin).
			// WithStdout(os.Stdout).
			WithStderr(os.Stderr).
			Build(ctx)

		machineOpts = append(machineOpts, firecracker.WithProcessRunner(cmd))
	}

	vmmCtx, vmmCancel := context.WithCancel(ctx)

	m, err := firecracker.NewMachine(vmmCtx, fcCfg, machineOpts...)
	if err != nil {
		vmmCancel()
		return nil, fmt.Errorf("failed creating machine: %s", err)
	}

	if err := m.Start(vmmCtx); err != nil {
		vmmCancel()
		return nil, fmt.Errorf("failed to start machine: %v", err)
	}

	log.WithField("ip", m.Cfg.NetworkInterfaces[0].StaticConfiguration.IPConfiguration.IPAddr.IP).Info("machine started")

	return &runningFirecracker{
		vmmCtx:    vmmCtx,
		vmmCancel: vmmCancel,
		vmmID:     vmmID,
		machine:   m,
		ip:        m.Cfg.NetworkInterfaces[0].StaticConfiguration.IPConfiguration.IPAddr.IP,
	}, nil
}

func installSignalHandlers() {
	go func() {
		// Clear some default handlers installed by the firecracker SDK:
		signal.Reset(os.Interrupt, syscall.SIGTERM, syscall.SIGQUIT)
		c := make(chan os.Signal, 1)
		signal.Notify(c, os.Interrupt, syscall.SIGTERM, syscall.SIGQUIT)

		for {
			switch s := <-c; {
			case s == syscall.SIGTERM || s == os.Interrupt:
				log.Printf("Caught signal: %s, requesting clean shutdown", s.String())
				deleteVMMSockets()
				os.Exit(0)
			case s == syscall.SIGQUIT:
				log.Printf("Caught signal: %s, forcing shutdown", s.String())
				deleteVMMSockets()
				os.Exit(0)
			}
		}
	}()
}

func (job benchJob) run(ctx context.Context, WarmVMs <-chan runningFirecracker) {
	log.WithField("job", job).Info("Handling job")

	err := queue.setjobReceived(ctx, job)
	if err != nil {
		log.WithError(err).Error("Could not set job received")
		queue.setjobFailed(ctx, job, agentExecRes{Error: err.Error()})
		return
	}

	// Get a ready-to-use microVM from the pool
	vm := <-WarmVMs

	// Defer cleanup of VM and VMM
	go func() {
		defer vm.vmmCancel()
		vm.machine.Wait(vm.vmmCtx)
	}()
	defer vm.shutDown()

	var reqJSON []byte

	reqJSON, err = json.Marshal(agentRunReq{
		ID:       job.ID,
		Language: job.Language,
		Code:     job.Code,
		Variant:  "TODO",
	})
	if err != nil {
		log.WithError(err).Error("Failed to marshal JSON request")
		queue.setjobFailed(ctx, job, agentExecRes{Error: err.Error()})
		return
	}

	err = queue.setjobRunning(ctx, job)
	if err != nil {
		log.WithError(err).Error("Could not set job running")
		queue.setjobFailed(ctx, job, agentExecRes{Error: err.Error()})
		return
	}

	var httpRes *http.Response
	var agentRes agentExecRes

	httpRes, err = http.Post("http://"+vm.ip.String()+":8080/run", "application/json", bytes.NewBuffer(reqJSON))
	if err != nil {
		log.WithError(err).Error("Failed to request execution to agent")
		queue.setjobFailed(ctx, job, agentExecRes{Error: err.Error()})
		return
	}
	json.NewDecoder(httpRes.Body).Decode(&agentRes)
	log.WithField("result", agentRes).Info("Job execution finished")
	if httpRes.StatusCode != 200 {
		log.WithFields(log.Fields{
			"httpRes":  httpRes,
			"agentRes": agentRes,
			"reqJSON":  string(reqJSON),
		}).Error("Failed to compile and run code")
		queue.setjobFailed(ctx, job, agentRes)
		return
	}

	err = queue.setjobResult(ctx, job, agentRes)
	if err != nil {
		queue.setjobFailed(ctx, job, agentExecRes{Error: err.Error()})
	}
}

func (vm runningFirecracker) shutDown() {
	log.WithField("ip", vm.ip).Info("stopping")
	vm.machine.StopVMM()
	err := os.Remove(vm.machine.Cfg.SocketPath)
	if err != nil {
		log.WithError(err).Error("Failed to delete firecracker socket")
	}
	err = os.Remove("/tmp/rootfs-" + vm.vmmID + ".ext4")
	if err != nil {
		log.WithError(err).Error("Failed to delete firecracker rootfs")
	}
}
