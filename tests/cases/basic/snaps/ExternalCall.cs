// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class ExternalCall : UdonSharpBehaviour
{
    public void _start()
    {
        System.Int32 __lcl_absValue_SystemInt32_0;
        UnityEngine.Transform __lcl_child_UnityEngineTransform_0;
        UnityEngine.Random.InitState(12345);
        __lcl_absValue_SystemInt32_0 = UnityEngine.Mathf.Abs(-99);
        this.gameObject.SetActive(true);
        __lcl_child_UnityEngineTransform_0 = this.transform.GetChild(0);
        return;
    }
}