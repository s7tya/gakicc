#include "test.h"

int t1() {
  struct {
    int a;
    int b;
  } x;
  x.a = 1;
  x.b = 2;
  return x.a;
}

int t2() {
  struct {
    int a;
    int b;
  } x;
  x.a = 1;
  x.b = 2;
  return x.b;
}

int t3() {
  struct {
    char a;
    int b;
    char c;
  } x;
  x.a = 1;
  x.b = 2;
  x.c = 3;
  x.a;
}

int t4() {
  struct {
    char a;
    int b;
    char c;
  } x;
  x.b = 1;
  x.b = 2;
  x.c = 3;
  x.b;
}

int t5() {
  struct {
    char a;
    int b;
    char c;
  } x;
  x.a = 1;
  x.b = 2;
  x.c = 3;
  x.c;
}

int t6() {
  struct {
    char a;
    char b;
  } x[3];
  char *p = x;
  p[0] = 0;
  x[0].a;
}

int t7() {
  struct {
    char a;
    char b;
  } x[3];
  char *p = x;
  p[1] = 1;
  x[0].b;
}

int t8() {
  struct {
    char a;
    char b;
  } x[3];
  char *p = x;
  p[2] = 2;
  x[1].a;
}

int t9() {
  struct {
    char a;
    char b;
  } x[3];
  char *p = x;
  p[3] = 3;
  x[1].b;
}

int t10() {
  struct {
    char a[3];
    char b[5];
  } x;
  char *p = &x;
  x.a[0] = 6;
  p[0];
}

int t11() {
  struct {
    char a[3];
    char b[5];
  } x;
  char *p = &x;
  x.b[0] = 7;
  p[3];
}

int t12() {
  struct {
    struct {
      char b;
    } a;
  } x;
  x.a.b = 6;
  x.a.b;
}

int t13() {
  struct {
    int a;
  } x;
  sizeof(x);
}

int t14() {
  struct {
    int a;
    int b;
  } x;
  sizeof(x);
}

int t15() {
  struct {
    int a, b;
  } x;
  sizeof(x);
}

int t16() {
  struct {
    int a[3];
  } x;
  sizeof(x);
}

int t17() {
  struct {
    int a;
  } x[4];
  sizeof(x);
}

int t18() {
  struct {
    int a[3];
  } x[2];
  sizeof(x);
}

int t19() {
  struct {
    char a;
    char b;
  } x;
  sizeof(x);
}

int t20() {
  struct {
    char a;
    int b;
  } x;
  sizeof(x);
}

int t21() {
  struct {
  } x;
  sizeof(x);
}

int t22() {
  struct {
    char a;
    int b;
  } x;
  sizeof(x);
}

int t23() {
  struct {
    int a;
    char b;
  } x;
  sizeof(x);
}

int main() {
  ASSERT(1, t1());
  ASSERT(2, t2());
  ASSERT(1, t3());
  ASSERT(2, t4());
  ASSERT(3, t5());
  ASSERT(0, t6());
  ASSERT(1, t7());
  ASSERT(2, t8());
  ASSERT(3, t9());
  ASSERT(6, t10());
  ASSERT(7, t11());
  ASSERT(6, t12());
  ASSERT(8, t13());
  ASSERT(16, t14());
  ASSERT(16, t15());
  ASSERT(24, t16());
  ASSERT(32, t17());
  ASSERT(48, t18());
  ASSERT(2, t19());
  ASSERT(0, t21());
  ASSERT(16, t22());
  ASSERT(16, t23());

  printf("OK\n");
  return 0;
}