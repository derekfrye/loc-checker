namespace Shapes;

public interface IWorker
{
    int Execute(int value)
    {
        return value + 1;
    }
}

public readonly record struct Accumulator(int Value)
{
    public static Accumulator operator +(Accumulator left, Accumulator right)
    {
        return new(left.Value + right.Value);
    }

    public static explicit operator int(Accumulator value)
    {
        return value.Value;
    }
}

public record PersonRecord(string Name)
{
    public string Greeting => $"hi {Name}";
}

public delegate int Processor(int input);

public sealed class Widget
{
    private readonly int[] _values = [2, 4, 6];

    public int this[int index]
    {
        get
        {
            return _values[index];
        }
    }

    public string Description => "widget";

    public int Compute(int value)
    {
        return value + this[1];
    }
}

public sealed class Notifier
{
    public event EventHandler? Changed;

    public event EventHandler Updated
    {
        add
        {
            Changed += value;
        }
        remove
        {
            Changed -= value;
        }
    }
}
