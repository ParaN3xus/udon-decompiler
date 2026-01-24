```csharp
using UdonSharp;
using UnityEngine;

public class Branching : UdonSharpBehaviour
{
    public int threshold = 10;
    public int value = 7;

    public int Evaluate()
    {
        if (value > threshold)
        {
            return 1;
        }
        else if (value == threshold)
        {
            return 0;
        }

        return -1;
    }
}
```

```json
{}
```
