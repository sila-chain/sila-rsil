// Docker Bake configuration for rsil images

variable "REGISTRY" {
  default = "ghcr.io/paradigmxyz"
}

variable "TAG" {
  default = "latest"
}

variable "BUILD_PROFILE" {
  default = "maxperf-symbols"
}

variable "FEATURES" {
  default = ""
}

// Git info for vergen (since .git is excluded from Docker context)
variable "VERGEN_GIT_SHA" {
  default = ""
}

variable "VERGEN_GIT_DESCRIBE" {
  default = ""
}

variable "VERGEN_GIT_DIRTY" {
  default = "false"
}

// Common settings for all targets
group "default" {
  targets = ["sila"]
}

group "nightly" {
  targets = ["sila", "sila-profiling"]
}

// Base target with shared configuration
target "_base" {
  dockerfile = "Dockerfile.depot"
  platforms  = ["linux/amd64", "linux/arm64"]
  args = {
    BUILD_PROFILE      = "${BUILD_PROFILE}"
    FEATURES           = "${FEATURES}"
    VERGEN_GIT_SHA     = "${VERGEN_GIT_SHA}"
    VERGEN_GIT_DESCRIBE = "${VERGEN_GIT_DESCRIBE}"
    VERGEN_GIT_DIRTY   = "${VERGEN_GIT_DIRTY}"
  }
  secret = [
    {
      type = "env"
      id   = "DEPOT_TOKEN"
    }
  ]
}
target "_base_profiling" {
  inherits = ["_base"]
  platforms  = ["linux/amd64"]
}

// Sila (rsil)
target "sila" {
  inherits = ["_base"]
  args = {
    BINARY        = "rsil"
    MANIFEST_PATH = "bin/rsil"
  }
  tags = ["${REGISTRY}/rsil:${TAG}"]
}

target "sila-profiling" {
  inherits = ["_base_profiling"]
  args = {
    BINARY        = "rsil"
    MANIFEST_PATH = "bin/rsil"
    BUILD_PROFILE = "profiling"
    FEATURES      = "jemalloc-prof"
  }
  tags = ["${REGISTRY}/rsil:nightly-profiling"]
}

// Hive test targets — single-platform, hivetests profile, tar output
target "_base_hive" {
  inherits  = ["_base"]
  platforms = ["linux/amd64"]
  args = {
    BUILD_PROFILE = "hivetests"
  }
}

variable "HIVE_OUTPUT_DIR" {
  default = "./artifacts"
}

target "hive" {
  inherits = ["_base_hive"]
  args = {
    BINARY        = "rsil"
    MANIFEST_PATH = "bin/rsil"
  }
  tags   = ["ghcr.io/sila-chain/sila-rsil:latest"]
  output = ["type=docker,dest=${HIVE_OUTPUT_DIR}/rsil_image.tar"]
}

// Kurtosis test target
target "kurtosis" {
  inherits  = ["_base_hive"]
  args = {
    BINARY        = "rsil"
    MANIFEST_PATH = "bin/rsil"
  }
  tags   = ["ghcr.io/sila-chain/sila-rsil:kurtosis-ci"]
  output = ["type=docker,dest=${HIVE_OUTPUT_DIR}/rsil_image.tar"]
}
