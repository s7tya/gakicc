#include "test.h"

int g1, g2[4];

int t1() {
  int a;
  a = 3;
  return a;
}
int t2() {
  int a = 3;
  return a;
}
int t3() {
  int a = 3;
  int z = 5;
  return a + z;
}
int t4() {
  int a = 3;
  return a;
}
int t5() {
  int a = 3;
  int z = 5;
  return a + z;
}
int t6() {
  int a;
  int b;
  a = b = 3;
  return a + b;
}
int t7() {
  int foo = 3;
  return foo;
}
int t8() {
  int foo123 = 3;
  int bar = 5;
  return foo123 + bar;
}

int t9() {
  int x;
  return sizeof(x);
}
int t10() {
  int x;
  return sizeof x;
}
int t11() {
  int *x;
  return sizeof(x);
}
int t12() {
  int x[4];
  return sizeof(x);
}
int t13() {
  int x[3][4];
  return sizeof(x);
}
int t14() {
  int x[3][4];
  return sizeof(*x);
}
int t15() {
  int x[3][4];
  return sizeof(**x);
}
int t16() {
  int x[3][4];
  return sizeof(**x) + 1;
}
int t17() {
  int x[3][4];
  return sizeof **x + 1;
}
int t18() {
  int x[3][4];
  return sizeof(**x + 1);
}
int t19() {
  int x = 1;
  return sizeof(x = 2);
}
int t20() {
  int x = 1;
  sizeof(x = 2);
  return x;
}

int t21() {
  g1 = 3;
  return g1;
}
int t22() {
  g2[0] = 0;
  g2[1] = 1;
  g2[2] = 2;
  g2[3] = 3;
  return g2[0];
}
int t23() {
  g2[0] = 0;
  g2[1] = 1;
  g2[2] = 2;
  g2[3] = 3;
  return g2[1];
}
int t24() {
  g2[0] = 0;
  g2[1] = 1;
  g2[2] = 2;
  g2[3] = 3;
  return g2[2];
}
int t25() {
  g2[0] = 0;
  g2[1] = 1;
  g2[2] = 2;
  g2[3] = 3;
  return g2[3];
}

int t26() {
  char x = 1;
  return x;
}
int t27() {
  char x = 1;
  char y = 2;
  return x;
}
int t28() {
  char x = 1;
  char y = 2;
  return y;
}
int t29() {
  char x;
  return sizeof(x);
}
int t30() {
  char x[10];
  return sizeof(x);
}

int t31() {
  int x = 2;
  {
    int x2 = 3;
  }
  return x;
}
int t32() {
  int x = 2;
  {
    int x2 = 3;
  }
  {
    int y = 4;
  }
  return x;
}
int t33() {
  int x = 2;
  {
    x = 3;
  }
  return x;
}

int main() {
  ASSERT(3, t1());
  ASSERT(3, t2());
  ASSERT(8, t3());

  ASSERT(3, t4());
  ASSERT(8, t5());
  ASSERT(6, t6());
  ASSERT(3, t7());
  ASSERT(8, t8());

  ASSERT(4, t9());
  ASSERT(4, t10());
  ASSERT(8, t11());
  ASSERT(16, t12());
  ASSERT(48, t13());
  ASSERT(16, t14());
  ASSERT(4, t15());
  ASSERT(5, t16());
  ASSERT(5, t17());
  ASSERT(4, t18());
  ASSERT(4, t19());
  ASSERT(1, t20());

  ASSERT(0, g1);
  ASSERT(3, t21());
  ASSERT(0, t22());
  ASSERT(1, t23());
  ASSERT(2, t24());
  ASSERT(3, t25());

  ASSERT(4, sizeof(g1));
  ASSERT(16, sizeof(g2));

  ASSERT(1, t26());
  ASSERT(1, t27());
  ASSERT(2, t28());

  ASSERT(1, t29());
  ASSERT(10, t30());

  ASSERT(2, t31());
  ASSERT(2, t32());
  ASSERT(3, t33());

  // TODO: block scope
  {
    void *x;
  }

  printf("OK\n");
  return 0;
}
