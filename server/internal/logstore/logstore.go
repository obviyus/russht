package logstore

import (
	"encoding/json"
	"fmt"
	"log"
	"net"
	"net/http"
	"os"
	"sync"
	"time"
)

type Log struct {
	Id 			int
	ServerID	string
	Content 	string
	IP 			string
	LogTime		time.Time
}

// LogStore is a simple in-memory database of tasks; TaskStore methods are
// safe to call concurrently.
type LogStore struct {
	sync.Mutex

	logs  map[int]Task
	nextId int
}

func New() *LogStore {
	ls := &LogStore{}
	ls.logs = make(map[int]Log)
	ls.nextId = 0

	return ls
}

// CreateLog creates a new log in the store.
func (ls *LogStore) CreateLog (text string, serverID string, IP string, logTime time.Time) int {
	ls.Lock()
	defer ls.Unlock()

	newLog := Log {
		Id: ls.nextId,
		ServerID: serverID,
		Content: text,
		IP: IP,
		LogTime: logTime,
	}

	ls.logs[ls.nextId] = newLog
	ls.nextId++

	return newLog.Id
}

// GetStore retrieves a task from the store, by id. If no such id exists, an
// error is returned.
func (ls *LogStore) GetStore(id int) (Store, error) {
	ls.Lock()
	defer ls.Unlock()

	l, ok := ls.logs[id]
	if ok {
		return l, nil
	} else {
		return Log{}, fmt.Errorf("log with id=%d not found", id)
	}
}

// DeleteStore deletes the task with the given id. If no such id exists, an error
// is returned.
func (ls *LogStore) DeleteStore(id int) error {
	ls.Lock()
	defer ls.Unlock()

	if _, ok := ls.logs[id]; !ok {
		return fmt.Errorf("log with id=%d not found", id)
	}

	delete(ls.logs, id)
	return nil
}

// DeleteAllStores deletes all tasks in the store.
func (ls *LogStore) DeleteAllStores() error {

}

// GetAllStores returns all the tasks in the store, in arbitrary order.
func (ls *LogStore) GetAllStores() []Store

// GetStoresByServer returns all the tasks that have the given due date, in
// arbitrary order.
func (ls *LogStore) GetStoresByDueDate(year int, month time.Month, day int) []Store


func NewLogServer() *logServer {
	store := logstore.New()
	return &logServer{store: store}
}

func (ls *logServer) logHandler(w http.ResponseWriter, req *http.Request) {
	if req.URL.Path == "/log/" {
		if req.Method == http.MethodPost {
			ls.createLogHandler(w, req)
		} else if  req.Method == http.MethodGet {
			ls.getAllLogHandlers(w, req)
		} else if req.Method == http.MethodDelete {
			ls.deleteAllLogsHandler(w, req)
		} else {
			http.Error(w, fmt.Sprintf("expected GET, DELETE or POST at /task/, got %v", req.Method), http.StatusMethodNotAllowed)
			return
		}
	}
}

func (ts *logServer) getAllLogHandlers(w http.ResponseWriter, req *http.Request) {
	log.Printf("handling get all logs at %s\n", req.URL.Path)

	allLogs := ls.store.GetAllLogs()
	js, err := json.Marshal(allLogs)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application-json")
	w.Write(js)
}

func main() {
	mux := http.NewServeMux()
	server := NewLogServer()

	mux.HandleFunc("/log", server.logHandler)
	mux.HandleFunc("/server", server.serverHandler)

	log.Fatal(http.ListenAndServe("localhost:"+os.Getenv("SERVERPORT"), mux))
}