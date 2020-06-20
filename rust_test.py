from ctypes import *
so_file = "target/debug/libencoder.so"
my_functions = CDLL(so_file)
print(type(my_functions))
assert my_functions.double(1) == 2
print('all good')
