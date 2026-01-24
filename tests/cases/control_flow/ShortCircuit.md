```csharp
using UdonSharp;
using UnityEngine;

public class ShortCircuit : UdonSharpBehaviour
{
    public int a = 10;
    public int b = 20;
    public int c = 30;

    public override void Interact()
    {
        if (a > 0 && b < 100)
        {
            Debug.Log("AND Check Passed");
        }
        if (a == 5 || b == 5)
        {
            Debug.Log("OR Check Passed");
        }
        if (a > b && (b > c || a == 10))
        {
            Debug.Log("Mixed Check Passed");
        }
    }
}
```

```json

```
