// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class ThisReference : UdonSharpBehaviour
{
    public void _start()
    {
        VRC.Udon.UdonBehaviour __lcl_currentBehaviour_VRCUdonUdonBehaviour_0;
        UnityEngine.GameObject __lcl_currentGo_UnityEngineGameObject_0;
        UnityEngine.Transform __lcl_currentTrans_UnityEngineTransform_0;
        __lcl_currentBehaviour_VRCUdonUdonBehaviour_0 = this;
        UnityEngine.Debug.Log("Behaviour: " + __lcl_currentBehaviour_VRCUdonUdonBehaviour_0.name);
        __lcl_currentGo_UnityEngineGameObject_0 = this.gameObject;
        UnityEngine.Debug.Log("GameObject: " + __lcl_currentGo_UnityEngineGameObject_0.name);
        __lcl_currentTrans_UnityEngineTransform_0 = this.transform;
        UnityEngine.Debug.Log(System.String.Concat("Transform Position: ", __lcl_currentTrans_UnityEngineTransform_0.position));
        return;
    }
}