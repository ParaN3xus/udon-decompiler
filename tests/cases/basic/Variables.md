```csharp
using UdonSharp;
using UnityEngine;

public class Variables : UdonSharpBehaviour
{
    public int count = 3;
    public float speed = 2.5f;
    public bool enabledFlag = true;

    private int _ticks;

    private void Update()
    {
        if (!enabledFlag)
        {
            return;
        }

        _ticks += 1;
        transform.Rotate(0f, speed, 0f);

        if (_ticks >= count)
        {
            enabledFlag = false;
        }
    }
}
```

```json
{}
```
