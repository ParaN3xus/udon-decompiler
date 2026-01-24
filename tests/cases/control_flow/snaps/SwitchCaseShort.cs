// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class SwitchCaseShort : UdonSharpBehaviour
{
    System.Int32 __0_mode__param = 0;

    public void __0_Run()
    {
        if (__0_mode__param == 0)
        {
            UnityEngine.Debug.Log("Mode Zero");
        }
        else
        {
            if (__0_mode__param == 1)
            {
                UnityEngine.Debug.Log("Mode One");
            }
            else
            {
                UnityEngine.Debug.Log("Default Mode");
            }
            return;
        }
    }
}