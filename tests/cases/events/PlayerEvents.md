```csharp
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class PlayerEvents : UdonSharpBehaviour
{
    public int joinCount;
    public int leaveCount;

    public override void OnPlayerJoined(VRCPlayerApi player)
    {
        joinCount += 1;
    }

    public override void OnPlayerLeft(VRCPlayerApi player)
    {
        leaveCount += 1;
    }
}
```

```json
{}
```
