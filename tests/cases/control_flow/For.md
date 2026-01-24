```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class For : UdonSharpBehaviour
{
    public void Start()
    {
        int[] numbers = new int[] { 1, 2, 3, 4, 5 };
        int total = 0;

        for (int i = 0; i < numbers.Length; i++)
        {
            total += numbers[i];
            Debug.Log(i);
        }

        Debug.Log(total);
    }
}
```

```json
ERROR: Compile Failed
```
