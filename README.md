### npxl viewer

Renders custom image format to png or simply displays it

#### the format
When writing programs to generate images it is super simple to create ppm files, unfortunately the ppm files must have space delimited pixel values which makes them take up a lot of storage space. To help mitigate this me and @nptnl created a specific format that allows you to use numbers of other bases for the pixel values, specific black and white files, and non space delimited pixel values to allow you to save room on storage space.

All numbers can only be stored as 1 character, different bases can be used to allow for different levels of depth.

example:
```
4 4
2 1
0101
1010
```

This creates an image that is 4 pixels by 4 pixels, black and white, and binary base pixel values (0 or 1)

additional example:
```
2 2
16 3
fff11e
9e219f
```

This creates an image that is 2 pixels by 2 pixels, hex values to store color, with rgb encoding
