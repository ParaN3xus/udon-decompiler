```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class While : UdonSharpBehaviour
{
    public void Start()
    {
        int counter = 0;
        int sum = 0;

        while (counter < 10)
        {
            sum += counter;
            counter++;

            if (sum > 20)
            {
                Debug.Log("Sum is growing");
            }
        }

        Debug.Log(sum);
    }
}
```

```json

```
