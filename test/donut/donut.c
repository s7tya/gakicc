int putchar();
void* memset();
int usleep();
int printf();

int m(int a, int b) { return (a * b + 5000) / 10000; }
void a(int* c, int* s, int d, int t) {
  int k = m(*c, d) - m(*s, t);
  int l = m(*s, d) + m(*c, t);
  *c = k;
  *s = l;
}

int main() {
  int z[1760];
  char b[1760];
  printf("\033[2J");
  int s = 10000;
  int q = s;
  int r = 0;
  int u = s;
  int v = 0;

  int count = 0;
  for (; count < 100; a(&q, &r, s - 8, 400), a(&u, &v, s - 2, 200)) {
    count = count + 1;

    memset(b, 32, 1760);
    memset(z, 0, 1760 * sizeof(q));
    int l = 0;
    int p = s;

    int i = 0;
    for (; i < 88; i = i + 1, a(&p, &l, 9974 + i % 2, 714)) {
      int w = 0;
      int e = s;

      int j = 0;
      for (; j < 314; j = j + 1, a(&e, &w, s - 2, 200)) {
        int f = p + 2 * s;
        int g = s * s / (m(m(w, f), r) + m(l, q) + 5 * s);
        int t = m(m(w, q), f) - m(l, r);
        int x = 40 + 30 * m(g, m(m(e, u), f) - m(t, v)) / s;
        int y = 12 + 15 * m(g, m(m(e, v), f) + m(t, u)) / s;
        int o = x + 80 * y;
        int N = 8 *
                (m(m(l, r) - m(m(w, q), p), u) - m(m(w, r), p) - m(l, q) -
                 m(m(e, v), p)) /
                s;
        if (y > 0 && g > z[o] && 22 > y && x > 0 && 80 > x) {
          z[o] = g;
          if (N >= 1) {
            b[o] = ".,-~:;=!*#$@"[N];
          } else {
            b[o] = "."[0];
          }
        }
      }
    }
    printf("\033[H");
    int k = 0;
    for (; k < 1761; k = k + 1) {
      if (k % 80) {
        putchar(b[k]);
      } else {
        putchar(10);
      }
    }
    // usleep(5 * s);
  }
  return 0;
}
