```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class InternalCall : UdonSharpBehaviour
{
    void Start()
    {
        Debug.Log(fibonacci(10).ToString());
    }

    int fibonacci(int n)
    {
        if (n <= 2)
        {
            return 1;
        }
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}
```

```json

```
