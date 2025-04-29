import numpy as np
import matplotlib.pyplot as plt
from scipy.optimize import curve_fit

x_data = np.array([0.001, 1.8681786817868178, 3.5004250042500424, 6.079130791307913, 8.62079620796208, 11.822098220982209, 14.770977709777098, 18.404364043640438, 24.22429224292243, 27.667156671566715, 30.243852438524385, 42.67484674846749, 43.379543795437954, 50.421834218342184, 49.873578735787355, 60.57922579225792, 59.62052620526205, 64.77774777747777])
y_data = np.array([0.001, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0])
def func(x, a, b,c):
    return -a / (x+c) + b
popt, pcov = curve_fit(func, x_data, y_data)
a, b, c = popt
a=int(a)
b=int(b)
c=int(c)
print(f"a = {a}, b = {b},c={c}")

x_fit = np.linspace(0, 100, 100)
plt.scatter(x_data, y_data, color='red')
plt.plot(x_fit, func(x_fit, a, b, c), label=f'lcas2bpd')
plt.legend()
plt.show()