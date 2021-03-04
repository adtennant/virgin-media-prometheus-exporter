target "ghaction-docker-meta" {}

target "build" {
  inherits = ["ghaction-docker-meta"]
  context = "./"
  dockerfile = "Dockerfile"
  platforms = ["linux/amd64", "linux/arm64", "linux/arm/v7"]
}