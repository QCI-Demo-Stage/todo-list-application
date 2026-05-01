package main

import (
	_ "embed"
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"
)

//go:embed api-spec.yaml
var openAPISpec []byte

//go:embed swagger/index.html
var swaggerIndexHTML []byte

func main() {
	port := 8000
	if p := os.Getenv("PORT"); p != "" {
		if n, err := strconv.Atoi(p); err == nil && n > 0 {
			port = n
		}
	}

	nodeEnv := os.Getenv("NODE_ENV")
	if nodeEnv == "" {
		nodeEnv = "staging"
	}
	docsEnabled := nodeEnv == "development" || nodeEnv == "staging"

	mux := http.NewServeMux()

	mux.HandleFunc("GET /health", handleHealth)

	if docsEnabled {
		mux.HandleFunc("GET /api-spec.yaml", handleAPISpec)
		mux.HandleFunc("GET /api-docs", handleSwaggerRedirect)
		mux.HandleFunc("GET /api-docs/", handleSwaggerUI)
	}

	addr := fmt.Sprintf(":%d", port)
	log.Printf("listening on http://localhost%s (NODE_ENV=%s)", addr, nodeEnv)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatal(err)
	}
}

func handleHealth(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(`{"status":"ok"}`))
}

func handleAPISpec(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "application/yaml; charset=utf-8")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(openAPISpec)
}

func handleSwaggerRedirect(w http.ResponseWriter, r *http.Request) {
	http.Redirect(w, r, "/api-docs/", http.StatusMovedPermanently)
}

func handleSwaggerUI(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(swaggerIndexHTML)
}
