```csharp
using UdonSharp;
using UnityEngine;

public class Loops : UdonSharpBehaviour
{
    public int limit = 5;
    public int sum;

    private void Start()
    {
        sum = 0;
        for (int i = 0; i < limit; i++)
        {
            sum += i;
        }
    }
}
```

```json
{}
```
