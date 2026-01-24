```csharp
using UdonSharp;
using UnityEngine;

public class HelloWorld : UdonSharpBehaviour
{
    public string message = "Hello, world!";

    private void Start()
    {
        Debug.Log(message);
    }
}
```

```json
{}
```
