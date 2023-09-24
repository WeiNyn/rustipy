import random

def test_function(a, b: str, c: int = 3, *args, **kwargs) -> random.Random:
    return random.Random()


def test_function2(a, b: str, *, d: int):
    return random.Random()


def test_function3(*kwoargs, case_sen=False):
    return random.Random()


class TestClass(random.Random):
    def test_method(self, a: int = 1, *, b: str = 2) -> random.Random:
        return random.Random()


class TestClass2(random.Random):
    def __init__(self):
        pass


test_var: str = 1
test_var2 = 2
TEST_CONST = test_function(1, "2")
