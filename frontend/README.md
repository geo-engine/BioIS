# BioIS Frontend

The frontend is an Angular application that provides an interactive web interface for users to visualize and interact with biodiversity indicators computed by the BioIS backend service.

## Development server

To start a local development server for the first time, run:

```bash
just \
    install-api-client-deps \
    install-frontend-deps \
    run-frontend
```

Once the server is running, open your browser and navigate to `http://localhost:4200/`. The application will automatically reload whenever you modify any of the source files.

Subsequent runs can be started with:

```bash
    just run-frontend
```

or

```bash
    just run
```

to start both the backend and frontend together.
