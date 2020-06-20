from ctypes import *
so_file = "target/debug/libencoder.so"
my_functions = CDLL(so_file)
print(type(my_functions))
assert my_functions.mul(2, 3) == 6
print('all good')
