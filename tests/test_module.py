def test_function(a: int = 1, b: str = 2) -> random.Random:
    return random.Random()

class TestClass(abc.ABC):
    def test_method(self, a: int = 1, b: str = 2) -> random.Random:
        return random.Random()
    
class TestClass2:
    def __init__(self):
        pass
    
    
test_var: str = 1
test_var2 = 2
TEST_CONST = test_function(1, "2")