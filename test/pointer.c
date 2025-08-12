#include "test.h"

int t1() {
  int x = 3;
  return *&x;
}
int t2() {
  int x = 3;
  int *y = &x;
  int **z = &y;
  return **z;
}
int t3() {
  int x = 3;
  int y = 5;
  return *(&x + 1);
}
int t4() {
  int x = 3;
  int y = 5;
  return *(&y - 1);
}
int t5() {
  int x = 3;
  int y = 5;
  return *(&x - (-1));
}
int t6() {
  int x = 3;
  int *y = &x;
  *y = 5;
  return x;
}
int t7() {
  int x = 3;
  int y = 5;
  *(&x + 1) = 7;
  return y;
}
int t8() {
  int x = 3;
  int y = 5;
  *(&y - 2 + 1) = 7;
  return x;
}
int t9() {
  int x = 3;
  return (&x + 2) - &x + 3;
}
int t10() {
  int x, y;
  x = 3;
  y = 5;
  return x + y;
}
int t11() {
  int x = 3, y = 5;
  return x + y;
}

int t12() {
  int x[2];
  int *y = x;
  *y = 3;
  return *x;
}
int t13() {
  int x[3];
  *x = 3;
  *(x + 1) = 4;
  *(x + 2) = 5;
  return *x;
}
int t14() {
  int x[3];
  *x = 3;
  *(x + 1) = 4;
  *(x + 2) = 5;
  return *(x + 1);
}
int t15() {
  int x[3];
  *x = 3;
  *(x + 1) = 4;
  *(x + 2) = 5;
  return *(x + 2);
}
int t16() {
  int x[2][3];
  int *y = x;
  *y = 0;
  return **x;
}
int t17() {
  int x[2][3];
  int *y = x;
  *(y + 1) = 1;
  return *(*x + 1);
}
int t18() {
  int x[2][3];
  int *y = x;
  *(y + 2) = 2;
  return *(*x + 2);
}
int t19() {
  int x[2][3];
  int *y = x;
  *(y + 3) = 3;
  return **(x + 1);
}
int t20() {
  int x[2][3];
  int *y = x;
  *(y + 4) = 4;
  return *(*(x + 1) + 1);
}
int t21() {
  int x[2][3];
  int *y = x;
  *(y + 5) = 5;
  return *(*(x + 1) + 2);
}
int t22() {
  int x[3];
  *x = 3;
  x[1] = 4;
  x[2] = 5;
  return *x;
}
int t23() {
  int x[3];
  *x = 3;
  x[1] = 4;
  x[2] = 5;
  return *(x + 1);
}
int t24() {
  int x[3];
  *x = 3;
  x[1] = 4;
  x[2] = 5;
  return *(x + 2);
}
int t25() {
  int x[3];
  *x = 3;
  x[1] = 4;
  x[2] = 5;
  return *(x + 2);
}
int t26() {
  int x[3];
  *x = 3;
  x[1] = 4;
  2 [x] = 5;
  return *(x + 2);
}
int t27() {
  int x[2][3];
  int *y = x;
  y[0] = 0;
  return x[0][0];
}
int t28() {
  int x[2][3];
  int *y = x;
  y[1] = 1;
  return x[0][1];
}
int t29() {
  int x[2][3];
  int *y = x;
  y[2] = 2;
  return x[0][2];
}
int t30() {
  int x[2][3];
  int *y = x;
  y[3] = 3;
  return x[1][0];
}
int t31() {
  int x[2][3];
  int *y = x;
  y[4] = 4;
  return x[1][1];
}
int t32() {
  int x[2][3];
  int *y = x;
  y[5] = 5;
  return x[1][2];
}

int main() {
  ASSERT(3, t1());
  ASSERT(3, t2());
  ASSERT(5, t3());
  ASSERT(3, t4());
  ASSERT(5, t5());
  ASSERT(5, t6());
  ASSERT(7, t7());
  ASSERT(7, t8());
  ASSERT(5, t9());
  ASSERT(8, t10());
  ASSERT(8, t11());
  ASSERT(3, t12());
  ASSERT(3, t13());
  ASSERT(4, t14());
  ASSERT(5, t15());
  ASSERT(0, t16());
  ASSERT(1, t17());
  ASSERT(2, t18());
  ASSERT(3, t19());
  ASSERT(4, t20());
  ASSERT(5, t21());
  ASSERT(3, t22());
  ASSERT(4, t23());
  ASSERT(5, t24());
  ASSERT(5, t25());
  ASSERT(5, t26());
  ASSERT(0, t27());
  ASSERT(1, t28());
  ASSERT(2, t29());
  ASSERT(3, t30());
  ASSERT(4, t31());
  ASSERT(5, t32());

  printf("OK\n");
  return 0;
}
