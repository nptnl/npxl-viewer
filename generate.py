res = "100 100"
base = "10 3"
data = ""
for y in range(100):
    for x in range(100):
        data += f"889"
    data += "\n"
with open("test.npxl", "w") as f:
    f.write(res + "\n" + base + "\n" + data)
