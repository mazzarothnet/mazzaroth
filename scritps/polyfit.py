import numpy as np
import matplotlib.pyplot as plt

# 生成示例数据
x = np.array([0, 1, 2, 3, 4])
y = np.array([1, 3, 2, 5, 7])
degree = 2  # 指定多项式次数

# 拟合系数
coefficients = np.polyfit(x, y, degree)
poly = np.poly1d(coefficients)  # 生成多项式函数

# 预测和绘图
x_fit = np.linspace(0, 4, 100)
plt.scatter(x, y, color='red')
plt.plot(x_fit, poly(x_fit), label=f'Degree {degree}')
plt.legend()
plt.show()