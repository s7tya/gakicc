#include "test.h"

int t1() {
  int i = 2;
  i += 5;
  return i;
}

int t2() {
  int i = 2;
  return i += 5;
}

int t3() {
  int i = 5;
  i -= 2;
  return i;
}

int t4() {
  int i = 5;
  return i -= 2;
}

int t5() {
  int i = 3;
  i *= 2;
  return i;
}

int t6() {
  int i = 3;
  return i *= 2;
}

int t7() {
  int i = 6;
  i /= 2;
  return i;
}

int t8() {
  int i = 6;
  return i /= 2;
}

int t9() {
  int i = 2;
  return i++;
}

int t10() {
  int i = 2;
  return i--;
}

int t11() {
  int i = 2;
  i++;
  return i;
}

int t12() {
  int i = 2;
  i--;
  return i;
}

int t13() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  return *p++;
}

int t14() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  *p--;
}

int t15() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  (*p++)--;
  a[0];
}

int t16() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  (*(p--))--;
  a[1];
}

int t17() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  (*p)--;
  a[2];
}

int t18() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  (*p)--;
  p++;
  *p;
}

int t19() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  (*p++)--;
  a[0];
}

int t20() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  (*p++)--;
  a[1];
}

int t21() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  (*p++)--;
  a[2];
}

int t22() {
  int a[3];
  a[0] = 0;
  a[1] = 1;
  a[2] = 2;
  int *p = a + 1;
  (*p++)--;
  *p;
}

int main() {
  ASSERT(0, 0);
  ASSERT(42, 42);
  ASSERT(21, 5 + 20 - 4);
  ASSERT(41, 12 + 34 - 5);
  ASSERT(47, 5 + 6 * 7);
  ASSERT(15, 5 * (9 - 6));
  ASSERT(4, (3 + 5) / 2);
  ASSERT(10, -10 + 20);
  ASSERT(10, - -10);
  ASSERT(10, - -+10);

  ASSERT(1, 10 % 3);
  ASSERT(3, 10 % 7);
  ASSERT(0, 5 % 5);

  ASSERT(0, 0 == 1);
  ASSERT(1, 42 == 42);
  ASSERT(1, 0 != 1);
  ASSERT(0, 42 != 42);

  ASSERT(1, 0 < 1);
  ASSERT(0, 1 < 1);
  ASSERT(0, 2 < 1);
  ASSERT(1, 0 <= 1);
  ASSERT(1, 1 <= 1);
  ASSERT(0, 2 <= 1);

  ASSERT(1, 1 > 0);
  ASSERT(0, 1 > 1);
  ASSERT(0, 1 > 2);
  ASSERT(1, 1 >= 0);
  ASSERT(1, 1 >= 1);
  ASSERT(0, 1 >= 2);

  ASSERT(7, t1());
  ASSERT(7, t2());
  ASSERT(3, t3());
  ASSERT(3, t4());
  ASSERT(6, t5());
  ASSERT(6, t6());
  ASSERT(3, t7());
  ASSERT(3, t8());

  ASSERT(2, t9());
  ASSERT(2, t10());
  ASSERT(3, t11());
  ASSERT(1, t12());
  ASSERT(1, t13());
  ASSERT(1, t14());

  ASSERT(0, t15());
  ASSERT(0, t16());
  ASSERT(2, t17());
  ASSERT(2, t18());

  ASSERT(0, t19());
  ASSERT(0, t20());
  ASSERT(2, t21());
  ASSERT(2, t22());

  printf("OK\n");
  return 0;
}