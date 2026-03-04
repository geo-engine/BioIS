# Containers

This directory contains Kubernetes manifests for running the BioIS backend and frontend in containers.
The manifests are designed for development and testing purposes, using a single Pod to host both services and a PVC for PostgreSQL data persistence.

## Running with Podman

Build images with Podman (from repository root):

```bash
just build-backend-container
just build-frontend-container
```

You can run them individually with `podman run`:

```bash
just run-frontend-container
just run-backend-container
```

## Running with Podman and Kubernetes manifests

Run locally using `podman play kube` with the provided manifest. The manifest now contains a single Pod and a PVC; Podman does not support Service objects.

Before applying, create a named Podman volume and use its mountpoint as the hostPath backing store for Postgres (no PersistentVolume resource is required):

```bash
# optionally, create a named podman volume
podman volume create biois-postgres

# build containers and run pods
just run-pod
```

Inspect running containers and ports:

```bash
podman ps
podman port <container-id-or-name>
```

Stop the deployed resources:

```bash
just down-pod
```

## Podman: Persistent volumes & Secrets

- The development `ConfigMap` and `Secret` are in `k8s/dev-config.yaml`.
  It contains the `biois-config` ConfigMap and `biois-postgres-secret` Secret used by the Pod.
  To override credentials, update that file or create a different Secret and apply it before the Pod.

Example: create an overriding secret and then apply manifests

```bash
kubectl create secret generic biois-postgres-secret --from-literal=POSTGRES_USER=youruser --from-literal=POSTGRES_PASSWORD=yourpass --from-literal=POSTGRES_DB=yourdb
```
