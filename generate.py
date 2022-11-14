res = "1000 1000"
base = "10 3"
data = ""
for y in range(1000):
    for x in range(1000):
        data += f"909"
    data += "\n"
with open("test.npxl", "w") as f:
    f.write(res + "\n" + base + "\n" + data)
