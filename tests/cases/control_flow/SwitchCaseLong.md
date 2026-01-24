```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class SwitchCaseLong : UdonSharpBehaviour
{
    public void Run(int id)
    {
        string itemName = "";

        switch (id)
        {
            case 101: itemName = "Apple"; break;
            case 102: itemName = "Banana"; break;
            case 103: itemName = "Cherry"; break;
            case 104: itemName = "Date"; break;
            case 105: itemName = "Elderberry"; break;
            case 106: itemName = "Fig"; break;
            case 107: itemName = "Grape"; break;
            case 108: itemName = "Honeydew"; break;
            case 109: itemName = "Pear"; break;
            case 110: itemName = "Peach"; break;
            default: itemName = "Unknown"; break;
        }

        Debug.Log(itemName);
    }
}
```

```json

```
