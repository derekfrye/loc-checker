namespace Demo;

public sealed class Calculator
{
    public int Add(int left, int right)
    {
        return left + right;
    }

    public int Multiply(int left, int right)
    {
        var result = 0;
        for (var index = 0; index < right; index++)
        {
            result += left;
        }

        return result;
    }

    public string Summary
    {
        get
        {
            return $"{Add(2, 3)}-{Multiply(3, 4)}";
        }
    }
}

public interface IGreeter
{
    string Greet(string name)
    {
        return $"hi {name}";
    }
}
