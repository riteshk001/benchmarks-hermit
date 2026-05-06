#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#include <sched.h>

// #define iterations 10000
#define MIN_MB 1
#define MAX_MB 1024
#define STEP_MB 1 // Step by 1 MB every iteration
#define FORKS_PER_STEP 100
#define SIZE_BYTE 1024 * 1024 // MB in bytes for ease of use

uint64_t now_ns() {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);

  return (uint64_t)ts.tv_sec * 1e9 + (uint64_t)ts.tv_nsec;
}

#if 0
int run_benchmark() {
  int result[iterations];
  double total;
  for (int i = 0; i < iterations; i++) {
    pid_t pid;

    uint64_t t1, t2, delta;

    t1 = now_ns();
    pid = fork();
    t2 = now_ns();

    if (pid < 0)
      return -1;
    else if (pid == 0)
      _exit(0);
    else {
      result[i] = t2 - t1;
      total += result[i];
      wait(NULL);
    }
  }
  double avg = total / iterations;
  printf("Average fork time: %.2f ns", avg);
  return 1;
}
#endif

void touch_memory(void *ptr, size_t size) {
  size_t page = sysconf(_SC_PAGESIZE);

  volatile char *p = (volatile char *)ptr;
  for (size_t i = 0; i < size; i += page) {
    p[i] = 1;
  }
}

int run_benchmark_progressive() {

  FILE *csv_file = fopen("benchmark_fork.csv", "w");

  fprintf(csv_file,
          "memory_size_mb,avg_fork_time,min_fork_time,max_fork_time\n");
  size_t size_mb;
  double avg_agg[1024];
  int z = 0;

  for (size_mb = MIN_MB; size_mb < MAX_MB; size_mb += STEP_MB) {
    size_t bytes = (size_t)size_mb * SIZE_BYTE;
    void *memory = (void *)malloc(bytes);
    if (!memory) {
      fprintf(stderr, "Failed to allocate memory %b MB\n", size_mb);
      break;
    }
    touch_memory(memory, bytes); // Touch every page in the range.

    double elapsed[FORKS_PER_STEP];
    uint64_t start, end;
    int success = 0;

    for (int i = 0; i < FORKS_PER_STEP; i++) {
      start = now_ns();
      // printf("start time now %lu\n", start);
      fflush(csv_file);
      pid_t pid = fork();
      end = now_ns();
      // printf("end time now %lu\n", end);

      if (pid == 0) {
        //_exit(0);
        char *argv[] = {"/bin/true", NULL};
        execv("/bin/true", argv);
        _exit(127);
      } else if (pid > 0) {
        elapsed[i] = end - start;
        success++;
        waitpid(pid, NULL, 0);
      } else {
        fprintf(stderr, "Failed to fork \n");
        elapsed[i] = -1;
        break;
      }
    }

    double total = 0.0, avg, min = 1e9, max = 0;

    for (int i = 0; i < FORKS_PER_STEP; i++) {
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
  // run_benchmark();
  run_benchmark_progressive();
  return 0;
}
