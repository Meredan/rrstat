#include <math.h>
#include <stdio.h>
#include <unistd.h>

// Volatile global to prevent optimization
volatile double sink = 0;

void hot_function() {
  // Heavy math work (sin/cos)
  for (int i = 0; i < 1000000; i++) {
    sink += sin(i) * cos(i);
  }
}

void warm_function() {
  // Lighter math work (just sin)
  for (int i = 0; i < 500000; i++) {
    sink += sin(i);
  }
}

int main() {
  printf("Test Target PID: %d\n", getpid());
  printf("Running CPU-intensive loop...\n");

  // Loop forever to give us time to attach
  while (1) {
    hot_function();
    warm_function();
  }
  return 0;
}
