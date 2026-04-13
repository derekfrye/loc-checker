using System;

namespace Demo;

public static class Program
{
    public static void Main()
    {
        Console.WriteLine(BuildMessage("world"));
        Console.WriteLine(LocalFormat("workspace"));

        string LocalFormat(string name)
        {
            var pieces = new[]
            {
                "hello",
                name,
                "from",
                "csharp",
            };
            return string.Join(" ", pieces);
        }
    }

    private static string BuildMessage(string name)
    {
        return $"hello {name}";
    }
}
