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

  printf("OK\n");
  return 0;
}