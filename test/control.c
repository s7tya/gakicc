#include "test.h"

/*
 * This is a block comment.
 */

int t1() {
  int x;
  if (0)
    x = 2;
  else
    x = 3;
  return x;
}
int t2() {
  int x;
  if (1 - 1)
    x = 2;
  else
    x = 3;
  return x;
}
int t3() {
  int x;
  if (1)
    x = 2;
  else
    x = 3;
  return x;
}
int t4() {
  int x;
  if (2 - 1)
    x = 2;
  else
    x = 3;
  return x;
}

int t5() {
  int i = 0;
  int j = 0;
  for (i = 0; i <= 10; i = i + 1) j = i + j;
  return j;
}
int t6() {
  int i = 0;
  while (i < 10) i = i + 1;
  return i;
}

int t7() { return 3; }
int t8() { return 5; }

int t9() {
  int i = 0;
  while (i < 10) i = i + 1;
  return i;
}
int t10() {
  int i = 0;
  int j = 0;
  while (i <= 10) {
    j = i + j;
    i = i + 1;
  }
  return j;
}

int t11() { return 1, 2, 3; }

int main() {
  ASSERT(3, t1());
  ASSERT(3, t2());
  ASSERT(2, t3());
  ASSERT(2, t4());

  ASSERT(55, t5());
  ASSERT(10, t6());

  ASSERT(3, t7());
  ASSERT(5, t8());

  ASSERT(10, t9());
  ASSERT(55, t10());

  ASSERT(3, t11());

  ASSERT(1, 0 || 1);
  ASSERT(1, 0 || (2 - 2) || 5);
  ASSERT(0, 0 || 0);
  ASSERT(0, 0 || (2 - 2));

  ASSERT(0, 0 || 0);
  ASSERT(1, 0 || 1);
  ASSERT(1, 1 || 0);
  ASSERT(1, 1 || 1);

  ASSERT(0, 0 && 1);
  ASSERT(0, (2 - 2) && 5);
  ASSERT(1, 1 && 5);

  ASSERT(0, 0 && 0);
  ASSERT(0, 0 && 1);
  ASSERT(0, 1 && 0);
  ASSERT(1, 1 && 1);

  printf("OK\n");
  return 0;
}
