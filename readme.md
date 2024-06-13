## Spin demo

1. install spin (nixpkgs) or ```curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash```
2. install prequesites for Rust: ```spin templates install --git https://github.com/fermyon/spin --update```
3. `rustup target add wasm32-wasi` (or `nix develop`)
4. `spin new` && `spin build` && `spin up`
5. Create github [token](https://github.com/fermyon/developer/blob/main/content/spin/v1/registry-tutorial.md) with package read/write/delete permissions
6. `spin registry push --build ghcr.io/joriatyben/spin-demo:0.1.0` 
7. create ghcr secret for Kubernetes like: `kubectl create secret docker-registry ghcr --docker-server ghcr.io --docker-username $user --docker-password $pass`
8. create deployment locally: `spin kube scaffold --from ghcr.io/$user/hello-spin:0.0.1 --out spinapp.yaml` 
9. `kubectl apply` to Kubernetes
10. `k port-forward svc/spin-demo 8888:80`
 
