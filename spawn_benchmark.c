#include <spawn.h>
#include <stdio.h>
#include <unistd.h>

#include <inttypes.h>
#include <sched.h>
#include <stdint.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <time.h>

#define ITERATIONS 10000
#define MIN_MB 1
#define MAX_MB 1024
#define STEP_MB 1 // Step by 1 MB every iteration
#define SPAWN_PER_STEP 100
#define SIZE_BYTE 1024 * 1024 // MB in bytes for ease of use

extern char **environ;
uint64_t now_ns() {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);

  return (uint64_t)ts.tv_sec * 1e9 + (uint64_t)ts.tv_nsec;
}

void touch_memory(void *ptr, size_t size) {
  size_t page = sysconf(_SC_PAGESIZE);

  volatile char *p = (volatile char *)ptr;
  for (size_t i = 0; i < size; i += page) {
    p[i] = 1;
  }
}

int run_benchmark_progressive() {
  // TODO: Write the main benchmark function for posix_spawn
  FILE *csv_file = fopen("benchmark_spawn.csv", "w");

  fprintf(csv_file,
          "memory_size_mb,avg_spawn_time,min_spawn_time,max_spawn_time\n");
  size_t size_mb;
  double avg_agg[1024];
  int z = 0;
  const char *prog = "/bin/true";
  char *argv[] = {"true", NULL};

  for (size_mb = MIN_MB; size_mb < MAX_MB; size_mb += STEP_MB) {
    size_t bytes = (size_t)size_mb * SIZE_BYTE;
    void *memory = (void *)malloc(bytes);
    if (!memory) {
      fprintf(stderr, "Failed to allocate memory %zu MB\n", size_mb);
      break;
    }
    touch_memory(memory, bytes); // Touch every page in the range.

    double elapsed[SPAWN_PER_STEP];
    uint64_t start, end;
    int success = 0;

    for (int i = 0; i < SPAWN_PER_STEP; i++) {
      start = now_ns();
      fflush(csv_file);
      // pid_t pid = fork();
      pid_t pid;
      // posix_spawn(&pid, prog, NULL, NULL, argv, NULL);

      if (posix_spawn(&pid, prog, NULL, NULL, argv, NULL) != 0) {
        fprintf(stderr, "pid creation failed, could not spawn new process \n");
        return -1;
      }
      waitpid(pid, NULL, 0);
      end = now_ns();
      elapsed[i] = end - start;
      success++;
    }

    double total = 0.0, avg, min = 1e9, max = 0;

    for (int i = 0; i < SPAWN_PER_STEP; i++) {
      if (elapsed[i] < 0)
        continue;
      total += elapsed[i];
      if (elapsed[i] < min)
        min = elapsed[i];
      if (elapsed[i] > max)
        max = elapsed[i];
    }
    avg = total / success;
    avg_agg[z] = avg;

    free(memory);

    fprintf(csv_file, "%zu,%.2f,%.2f,%.2f\n", size_mb, avg_agg[z], min, max);
  }
  fclose(csv_file);
  return 0;
}

int main() {
  run_benchmark_progressive();
  return 0;
}
