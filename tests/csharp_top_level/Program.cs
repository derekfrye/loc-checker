using System;

var values = new[] { 1, 2, 3 };
var total = 0;

foreach (var value in values)
{
    total += value;
}

Console.WriteLine(total);
