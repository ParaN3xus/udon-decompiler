<!-- ci: skip-compile -->

```csharp
﻿
using UdonSharp;
using UnityEngine;
using VRC.SDK3.Components;
using VRC.SDK3.Data;
using VRC.SDKBase;
using VRC.Udon.Common.Interfaces;
using Utilities = VRC.SDKBase.Utilities;

#pragma warning disable IDE0044
#pragma warning disable IDE0090, IDE1006

namespace QvPen.UdonScript
{
    [AddComponentMenu("")]
    [DefaultExecutionOrder(10)]
    [UdonBehaviourSyncMode(BehaviourSyncMode.NoVariableSync)]
    public class QvPen_Pen : UdonSharpBehaviour
    {
        public const string version = "v3.3.14";

        #region Field

        [Header("Pen")]
        [SerializeField]
        private TrailRenderer trailRenderer;
        [SerializeField]
        private LineRenderer inkPrefab;

        [SerializeField]
        private Transform inkPosition;
        [SerializeField]
        private Transform inkPositionChild;

        [SerializeField]
        private Transform inkPoolRoot;
        private Transform inkPool;
        private Transform inkPoolSynced;
        private Transform inkPoolNotSynced;

        private bool allowCallPen;
        public bool AllowCallPen => allowCallPen;

        private QvPen_Manager manager;

        [SerializeField]
        private QvPen_LateSync syncer;

        [Header("Pointer")]
        [SerializeField]
        private Transform pointer;

        private bool _isCheckedPointerRadius = false;
        private float _pointerRadius = 0f;
        private float pointerRadius
        {
            get
            {
                if (_isCheckedPointerRadius)
                {
                    return _pointerRadius;
                }
                else
                {
                    var sphereCollider = pointer.GetComponent<SphereCollider>();
                    sphereCollider.enabled = false;
                    var s = pointer.lossyScale;
                    _pointerRadius = Mathf.Max(0.01f, Mathf.Min(s.x, s.y, s.z)) * sphereCollider.radius;
                    _isCheckedPointerRadius = true;
                    return _pointerRadius;
                }
            }
        }
        [SerializeField]
        private float _pointerRadiusMultiplierForDesktop = 3f;
        private float pointerRadiusMultiplierForDesktop => isUserInVR ? 1f : Mathf.Abs(_pointerRadiusMultiplierForDesktop);
        [SerializeField]
        private Material pointerMaterialNormal;
        [SerializeField]
        private Material pointerMaterialActive;

        [Header("Screen")]
        [SerializeField]
        private Canvas screenOverlay;
        [SerializeField]
        private Renderer marker;

        [Header("Other")]
        [SerializeField]
        private bool canBeErasedWithOtherPointers = true;

        private bool enabledLateSync = true;

        private MeshCollider _inkPrefabCollider;
        private MeshCollider inkPrefabCollider
            => Utilities.IsValid(_inkPrefabCollider)
                ? _inkPrefabCollider : (_inkPrefabCollider = inkPrefab.GetComponentInChildren<MeshCollider>(true));
        //private GameObject lineInstance;

        private bool isUser;
        public bool IsUser => isUser;

        // Components
        private VRCPickup _pickup;
        private VRCPickup pickup
            => Utilities.IsValid(_pickup)
                ? _pickup : (_pickup = (VRCPickup)GetComponent(typeof(VRCPickup)));

        private VRCObjectSync _objectSync;
        private VRCObjectSync objectSync
            => Utilities.IsValid(_objectSync)
                ? _objectSync : (_objectSync = (VRCObjectSync)GetComponent(typeof(VRCObjectSync)));

        [Header("ObjectSync")]
        [SerializeField]
        private UdonSharpBehaviour _alternativeObjectSync;
        [SerializeField]
        private string _respawnEventName = "Respawn";

        // PenManager
        private QvPen_PenManager penManager;

        // Ink
        private int inkMeshLayer;
        private int inkColliderLayer;
        private int inkColliderLayerMask;
        private const float followSpeed = 32f;

        // Pointer
        private bool isPointerEnabled;
        private Renderer pointerRenderer;

        // Double click
        private bool useDoubleClick = true;
        private const float clickTimeInterval = 0.2f;
        private float prevClickTime;
        private float clickPosInterval = 0.01f; // Default: Quest
        private Vector3 prevClickPos;

        // State
        private QvPen_Pen_State currentState = QvPen_Pen_State.PenIdle;

        // Sync state
        [System.NonSerialized]
        public QvPen_Pen_SyncState currentSyncState = QvPen_Pen_SyncState.Idle;

        // Ink pool
        public const string inkPoolRootName = "QvPen_Objects";
        public const string inkPoolName = "InkPool";
        private int penId;
        public Vector3 penIdVector { get; private set; }
        private string penIdString;

        private const string inkPrefix = "Ink";
        private float inkWidth;
        bool isRoundedTrailShader = false;
        MaterialPropertyBlock propertyBlock;

        private VRCPlayerApi _localPlayer;
        private VRCPlayerApi localPlayer => _localPlayer ?? (_localPlayer = Networking.LocalPlayer);

        private bool _isCheckedLocalPlayerId = false;
        private int _localPlayerId;
        private int localPlayerId
            => _isCheckedLocalPlayerId
                ? _localPlayerId
                : (_isCheckedLocalPlayerId = Utilities.IsValid(localPlayer))
                    ? _localPlayerId = localPlayer.playerId
                    : 0;

        private bool _isCheckedLocalPlayerIdVector = false;
        private Vector3 _localPlayerIdVector;
        private Vector3 localPlayerIdVector
        {
            get
            {
                if (_isCheckedLocalPlayerIdVector)
                    return _localPlayerIdVector;

                _localPlayerIdVector = QvPenUtilities.GetPlayerIdVector(localPlayerId);
                _isCheckedLocalPlayerIdVector = true;
                return _localPlayerIdVector;
            }
        }

        private bool _isCheckedIsUserInVR = false;
        private bool _isUserInVR;
        private bool isUserInVR => _isCheckedIsUserInVR
            ? _isUserInVR
            : (_isCheckedIsUserInVR = Utilities.IsValid(localPlayer)) && (_isUserInVR = localPlayer.IsUserInVR());

        //private long TimeStamp => ((System.DateTimeOffset)Networking.GetNetworkDateTime()).ToUnixTimeSeconds();

        private readonly DataList localInkHistory = new DataList();

        #endregion Field

        public void _Init(QvPen_PenManager penManager)
        {
            this.penManager = penManager;
            _UpdateInkData();

            var inkPoolRootGO = GameObject.Find($"/{inkPoolRootName}");
            if (Utilities.IsValid(inkPoolRootGO))
            {
                inkPoolRoot.gameObject.SetActive(false);
                inkPoolRoot = inkPoolRootGO.transform;
            }
            else
            {
                inkPoolRoot.name = inkPoolRootName;
                QvPenUtilities.SetParentAndResetLocalTransform(inkPoolRoot, null);
                inkPoolRoot.SetAsFirstSibling();
                inkPoolRoot.gameObject.SetActive(true);
#if !UNITY_EDITOR
                Log($"{nameof(QvPen)} {version}");
#endif
            }

            inkPool = syncer.transform;
            QvPenUtilities.SetParentAndResetLocalTransform(inkPool, inkPoolRoot);

            var unique = Networking.GetUniqueName(gameObject);
            penId = string.IsNullOrEmpty(unique) ? 0 : unique.GetHashCode();
            penIdVector = QvPenUtilities.Int32ToVector3(penId);
            penIdString = $"0x{(int)penIdVector.x:x2}{(int)penIdVector.y:x3}{(int)penIdVector.z:x3}";
            inkPool.name = $"{inkPoolName} ({penIdString})";

            allowCallPen = penManager.AllowCallPen;

            manager = inkPoolRoot.GetComponent<QvPen_Manager>();
            manager.Register(penId, this);

            syncer._RegisterPen(this);

            inkPoolSynced = syncer.InkPoolSynced;
            inkPoolNotSynced = syncer.InkPoolNotSynced;

#if !UNITY_EDITOR
            Log($"QvPen ID: {penIdString}");
#endif

            pickup.InteractionText = nameof(QvPen);
            pickup.UseText = "Draw";

            pointerRenderer = pointer.GetComponent<Renderer>();
            pointer.gameObject.SetActive(false);
            pointer.transform.localScale *= pointerRadiusMultiplierForDesktop;

            marker.transform.localScale = Vector3.one * inkWidth;

#if UNITY_STANDALONE
            if (isUserInVR)
                clickPosInterval = 0.005f;
            else
                clickPosInterval = 0.001f;
#endif
        }

        public void _UpdateInkData()
        {
            inkWidth = penManager.inkWidth;
            inkMeshLayer = penManager.inkMeshLayer;
            inkColliderLayer = penManager.inkColliderLayer;
            inkColliderLayerMask = 1 << inkColliderLayer;

            inkPrefab.gameObject.layer = inkMeshLayer;
            trailRenderer.gameObject.layer = inkMeshLayer;
            inkPrefabCollider.gameObject.layer = inkColliderLayer;

#if UNITY_STANDALONE
            var material = penManager.pcInkMaterial;

            inkPrefab.material = material;
            trailRenderer.material = material;

            if (Utilities.IsValid(material))
            {
                var shader = material.shader;
                if (Utilities.IsValid(shader))
                {
                    isRoundedTrailShader = shader == penManager.roundedTrailShader;
                    isRoundedTrailShader |= shader.name.Contains("rounded_trail");
                }
            }

            if (isRoundedTrailShader)
            {
                inkPrefab.widthMultiplier = 0f;
                propertyBlock = new MaterialPropertyBlock();
                inkPrefab.GetPropertyBlock(propertyBlock);
                propertyBlock.SetFloat("_Width", inkWidth);
                inkPrefab.SetPropertyBlock(propertyBlock);

                trailRenderer.widthMultiplier = 0f;
                propertyBlock.Clear();
                trailRenderer.GetPropertyBlock(propertyBlock);
                propertyBlock.SetFloat("_Width", inkWidth);
                trailRenderer.SetPropertyBlock(propertyBlock);
            }
            else
            {
                inkPrefab.widthMultiplier = inkWidth;
                trailRenderer.widthMultiplier = inkWidth;
            }
#else
            var material = penManager.questInkMaterial;
            inkPrefab.material = material;
            trailRenderer.material = material;
            inkPrefab.widthMultiplier = inkWidth;
            trailRenderer.widthMultiplier = inkWidth;
#endif

            inkPrefab.colorGradient = penManager.colorGradient;
            trailRenderer.colorGradient = penManager.colorGradient;

            surftraceMask = penManager.surftraceMask;
        }

        public bool _CheckId(Vector3 idVector)
            => idVector == penIdVector;

        #region Data protocol

        #region Base

        // Footer element
        public const int FOOTER_ELEMENT_DATA_INFO = 0;
        public const int FOOTER_ELEMENT_PEN_ID = 1;
        public const int FOOTER_ELEMENT_INK_ID = 2;
        public const int FOOTER_ELEMENT_OWNER_ID = 3;

        public const int FOOTER_ELEMENT_ANY_LENGTH = 4;

        public const int FOOTER_ELEMENT_DRAW_INK_INFO = 4;
        public const int FOOTER_ELEMENT_DRAW_LENGTH = 5;

        public const int FOOTER_ELEMENT_ERASE_LENGTH = 4;
        public const int FOOTER_ELEMENT_ERASE_USER_INK_LENGTH = 4;

        private static int GetFooterSize(QvPen_Pen_Mode mode)
        {
            switch (mode)
            {
                case QvPen_Pen_Mode.Draw: return FOOTER_ELEMENT_DRAW_LENGTH;
                case QvPen_Pen_Mode.Erase: return FOOTER_ELEMENT_ERASE_LENGTH;
                case QvPen_Pen_Mode.EraseUserInk: return FOOTER_ELEMENT_ERASE_USER_INK_LENGTH;
                case QvPen_Pen_Mode.Any: return FOOTER_ELEMENT_ANY_LENGTH;
                case QvPen_Pen_Mode.None: return 0;
                default: return 0;
            }
        }

        #endregion

        private static Vector3 GetData(Vector3[] data, int index)
            => data != null && data.Length > index ? data[data.Length - 1 - index] : default;

        private static void SetData(Vector3[] data, int index, Vector3 element)
        {
            if (data != null && data.Length > index)
                data[data.Length - 1 - index] = element;
        }

        private static QvPen_Pen_Mode GetMode(Vector3[] data)
            => data != null && data.Length > 0 ? (QvPen_Pen_Mode)(int)GetData(data, FOOTER_ELEMENT_DATA_INFO).y : QvPen_Pen_Mode.None;

        private static int GetFooterLength(Vector3[] data)
            => data != null && data.Length > 0 ? Mathf.Clamp((int)GetData(data, FOOTER_ELEMENT_DATA_INFO).z, 0, data.Length) : 0;

        #endregion

        #region Unity events

        #region Screen mode
#if UNITY_STANDALONE
        private VRCPlayerApi.TrackingData headTracking;
        private Vector3 headPos, center;
        private Quaternion headRot;
        private Vector2 _wh, wh, clampWH;
        // Wait for Udon Vector2.Set() bug fix
        private /*readonly*/ Vector2 mouseDelta = new Vector2();
        private float ratio, scalar;

        private float sensitivity = 0.75f;
        private bool isScreenMode = false;
        private void Update()
        {
            if (isUserInVR || !isUser)
                return;

            if (Input.GetKeyUp(KeyCode.Tab))
            {
                ExitScreenMode();
            }

            if (!Input.anyKey)
                return;

            if (Input.GetKeyDown(KeyCode.Backspace))
            {
                _UndoDraw();
            }
            else if (Input.GetKeyDown(KeyCode.Tab))
            {
                EnterScreenMode();
            }
            else if (Input.GetKey(KeyCode.Tab))
            {
                if (Input.GetKeyDown(KeyCode.Delete))
                {
                    penManager.SendCustomNetworkEvent(NetworkEventTarget.All, nameof(QvPen_PenManager.Clear));
                }
                else if (Input.GetKey(KeyCode.Home))
                {
                    penManager.SendCustomNetworkEvent(NetworkEventTarget.All, nameof(QvPen_PenManager.Respawn));
                }
                else if (Input.GetKey(KeyCode.UpArrow))
                {
                    sensitivity = Mathf.Min(sensitivity + 0.001f, 5.0f);
                    Log($"Sensitivity -> {sensitivity:f3}");
                }
                else if (Input.GetKey(KeyCode.DownArrow))
                {
                    sensitivity = Mathf.Max(sensitivity - 0.001f, 0.01f);
                    Log($"Sensitivity -> {sensitivity:f3}");
                }
            }
        }

        private void EnterScreenMode()
        {
            isScreenMode = true;

            marker.enabled = true;

            _wh = Vector2.zero;
            screenOverlay.gameObject.SetActive(true);
            wh = screenOverlay.GetComponent<RectTransform>().rect.size;
            screenOverlay.gameObject.SetActive(false);
            clampWH = wh / (2f * 1920f * 0.98f);
            ratio = 2f * 1080f / wh.y;
        }

        private void ExitScreenMode()
        {
            isScreenMode = false;

            if (!isSurftraceMode)
                marker.enabled = false;

            SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenIdle));

            inkPositionChild.SetLocalPositionAndRotation(Vector3.zero, Quaternion.identity);
            trailRenderer.transform.SetPositionAndRotation(inkPositionChild.position, inkPositionChild.rotation);
        }
#endif
        #endregion Screen mode

        private void LateUpdate()
        {
            if (!isHeld)
                return;

#if UNITY_STANDALONE
            if (!isUserInVR && isUser && Input.GetKey(KeyCode.Tab))
            {
                headTracking = localPlayer.GetTrackingData(VRCPlayerApi.TrackingDataType.Head);
                headPos = headTracking.position;
                headRot = headTracking.rotation;

                center = headRot * Vector3.forward * Vector3.Dot(headRot * Vector3.forward, transform.position - headPos);
                scalar = ratio * Vector3.Dot(headRot * Vector3.forward, center);
                center += headPos;

                // Wait for Udon Vector2.Set() bug fix
                // mouseDelta.Set(Input.GetAxis("Mouse X"), Input.GetAxis("Mouse Y"));
                {
                    mouseDelta.x = Input.GetAxis("Mouse X");
                    mouseDelta.y = Input.GetAxis("Mouse Y");
                }
                _wh += sensitivity * Time.deltaTime * mouseDelta;
                _wh = Vector2.Min(Vector2.Max(_wh, -clampWH), clampWH);

                inkPositionChild.SetPositionAndRotation(center + headRot * _wh * scalar, headRot);
            }
#endif

            if (isSurftraceMode)
            {
                Vector3 inkPositionPosition;
#if UNITY_STANDALONE
                if (isScreenMode)
                    inkPositionPosition = inkPositionChild.position;
                else
#endif
                    inkPositionPosition = inkPosition.position;

                var closestPoint = surftraceTarget.ClosestPoint(inkPositionPosition);
                var distance = Vector3.Distance(closestPoint, inkPositionPosition);

#if UNITY_STANDALONE
                inkPositionChild.position = Vector3.MoveTowards(closestPoint, inkPositionPosition, inkWidth / 1.999f);
#else
                inkPositionChild.position = Vector3.MoveTowards(closestPoint, inkPositionPosition, inkWidth / 1.9f);
#endif

                if (distance > surftraceMaxDistance)
                    ExitSurftraceMode();
            }

            if (!isPointerEnabled)
            {
                if (isUser)
                {
                    var deltaDistance = Time.deltaTime * followSpeed;
                    trailRenderer.transform.SetPositionAndRotation(
                        Vector3.Lerp(trailRenderer.transform.position, inkPositionChild.position, deltaDistance),
                        Quaternion.Lerp(trailRenderer.transform.rotation, inkPositionChild.rotation, deltaDistance));
                }
                else
                {
                    trailRenderer.transform.SetPositionAndRotation(inkPositionChild.position, inkPositionChild.rotation);
                }
            }
        }

        private readonly Collider[] results4 = new Collider[4];
        public override void PostLateUpdate()
        {
            if (!isUser)
                return;

            if (isPointerEnabled)
            {
                var count = Physics.OverlapSphereNonAlloc(pointer.position, pointerRadius, results4, inkColliderLayerMask, QueryTriggerInteraction.Ignore);
                for (var i = 0; i < count; i++)
                {
                    var other = results4[i];

                    Transform t1, t2, t3;

                    if (Utilities.IsValid(other)
                        && Utilities.IsValid(t1 = other.transform.parent)
                        && Utilities.IsValid(t2 = t1.parent))
                    {
                        if (canBeErasedWithOtherPointers
                          ? Utilities.IsValid(t3 = t2.parent) && t3.parent == inkPoolRoot
                          : t2.parent == inkPool
                        )
                        {
                            var lineRenderer = other.GetComponentInParent<LineRenderer>();
                            if (Utilities.IsValid(lineRenderer) && lineRenderer.positionCount > 0)
                            {
                                SendEraseInk(lineRenderer.gameObject);
                            }
                        }
                    }

                    results4[i] = null;
                }
            }
        }

        // Surftrace mode
        private bool useSurftraceMode = true;

        private const float surftraceMaxDistance = 1f;
        private const float surftraceEnterDistance = 0.05f;

        private int surftraceMask = ~0;

        private Collider surftraceTarget = null;
        private bool isSurftraceMode => surftraceTarget;

        private void OnTriggerEnter(Collider other)
        {
            if (isUser && useSurftraceMode && Utilities.IsValid(other) && !other.isTrigger)
            {
                if ((1 << other.gameObject.layer & surftraceMask) == 0)
                    return;

                //if (other.GetType().IsSubclassOf(typeof(MeshCollider)) && !((MeshCollider)other).convex)
                if (other.GetType() == typeof(MeshCollider) && !((MeshCollider)other).convex)
                    return;

                var distance = Vector3.Distance(other.ClosestPoint(inkPosition.position), inkPosition.position);
                if (distance < surftraceEnterDistance)
                    EnterSurftraceMode(other);
            }
        }

        private void EnterSurftraceMode(Collider target)
        {
            surftraceTarget = target;
            marker.enabled = true;
        }

        private void ExitSurftraceMode()
        {
            surftraceTarget = null;

#if UNITY_STANDALONE
            if (!isScreenMode)
#endif
                marker.enabled = false;

            SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenIdle));

            inkPositionChild.SetLocalPositionAndRotation(Vector3.zero, Quaternion.identity);
            trailRenderer.transform.SetPositionAndRotation(inkPositionChild.position, inkPositionChild.rotation);
        }

        #endregion Unity events

        #region VRChat events

        public override void OnPickup()
        {
            isUser = true;

            manager.SetLastUsedPen(this);

            penManager.OnPenPickup();

            penManager._TakeOwnership();
            penManager.SendCustomNetworkEvent(NetworkEventTarget.All, nameof(QvPen_PenManager.StartUsing));

            SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenIdle));
        }

        public override void OnDrop()
        {
            isUser = false;

            penManager.OnPenDrop();

            penManager.SendCustomNetworkEvent(NetworkEventTarget.All, nameof(QvPen_PenManager.EndUsing));

            SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenIdle));

            penManager._ClearSyncBuffer();

#if UNITY_STANDALONE
            ExitScreenMode();
#endif
            ExitSurftraceMode();
        }

        public override void OnPickupUseDown()
        {
            if (useDoubleClick
             && Time.time - prevClickTime < clickTimeInterval
             && Vector3.Distance(inkPosition.position, prevClickPos) < clickPosInterval
            )
            {
                prevClickTime = 0f;
                switch (currentState)
                {
                    case QvPen_Pen_State.PenIdle:
                        if (Vector3.Distance(inkPosition.position, prevClickPos) > 0f)
                            _UndoDraw();

                        SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToEraseIdle));
                        break;
                    case QvPen_Pen_State.EraserIdle:
                        SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenIdle));
                        break;
                    default:
                        Error($"Unexpected state : {currentState.ToStr()} at {nameof(OnPickupUseDown)} Double Clicked");
                        break;
                }
            }
            else
            {
                prevClickTime = Time.time;
                prevClickPos = inkPosition.position;
                switch (currentState)
                {
                    case QvPen_Pen_State.PenIdle:
                        SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenUsing));
                        break;
                    case QvPen_Pen_State.EraserIdle:
                        SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToEraseUsing));
                        InteractOtherUdon();
                        break;
                    default:
                        Error($"Unexpected state : {currentState.ToStr()} at {nameof(OnPickupUseDown)}");
                        break;
                }
            }
        }

        public override void OnPickupUseUp()
        {
            switch (currentState)
            {
                case QvPen_Pen_State.PenUsing:
                    SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenIdle));
                    break;
                case QvPen_Pen_State.EraserUsing:
                    SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToEraseIdle));
                    break;
                case QvPen_Pen_State.PenIdle:
                    Log($"Change state : {nameof(QvPen_Pen_State.EraserIdle)} to {currentState.ToStr()}");
                    break;
                case QvPen_Pen_State.EraserIdle:
                    Log($"Change state : {nameof(QvPen_Pen_State.PenIdle)} to {currentState.ToStr()}");
                    break;
                default:
                    Error($"Unexpected state : {currentState.ToStr()} at {nameof(OnPickupUseUp)}");
                    break;
            }
        }

        public void _SetUseDoubleClick(bool value)
        {
            useDoubleClick = value;

            if (isUser)
                SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenIdle));
        }

        public void _SetEnabledLateSync(bool value)
        {
            enabledLateSync = value;
        }

        public void _SetUseSurftraceMode(bool value)
        {
            useSurftraceMode = value;

            if (isUser)
                SendCustomNetworkEvent(NetworkEventTarget.All, nameof(ChangeStateToPenIdle));
        }

        private void OnEnable()
        {
            if (Utilities.IsValid(inkPool))
                inkPool.gameObject.SetActive(true);
        }

        private void OnDisable()
        {
            if (Utilities.IsValid(inkPool))
                inkPool.gameObject.SetActive(false);
        }

        private void OnDestroy()
        {
            _Clear();

            if (Utilities.IsValid(inkPool))
                Destroy(inkPool.gameObject);
        }

        #endregion

        #region ChangeState

        public void ChangeStateToPenIdle()
        {
            switch (currentState)
            {
                case QvPen_Pen_State.PenUsing:
                    FinishDrawing();
                    break;
                case QvPen_Pen_State.EraserIdle:
                    ChangeToPen();
                    break;
                case QvPen_Pen_State.EraserUsing:
                    DisablePointer();
                    ChangeToPen();
                    break;
            }
            currentState = QvPen_Pen_State.PenIdle;
        }

        public void ChangeStateToPenUsing()
        {
            switch (currentState)
            {
                case QvPen_Pen_State.PenIdle:
                    StartDrawing();
                    break;
                case QvPen_Pen_State.EraserIdle:
                    ChangeToPen();
                    StartDrawing();
                    break;
                case QvPen_Pen_State.EraserUsing:
                    DisablePointer();
                    ChangeToPen();
                    StartDrawing();
                    break;
            }
            currentState = QvPen_Pen_State.PenUsing;
        }

        public void ChangeStateToEraseIdle()
        {
            switch (currentState)
            {
                case QvPen_Pen_State.PenIdle:
                    ChangeToEraser();
                    break;
                case QvPen_Pen_State.PenUsing:
                    FinishDrawing();
                    ChangeToEraser();
                    break;
                case QvPen_Pen_State.EraserUsing:
                    DisablePointer();
                    break;
            }
            currentState = QvPen_Pen_State.EraserIdle;
        }

        public void ChangeStateToEraseUsing()
        {
            switch (currentState)
            {
                case QvPen_Pen_State.PenIdle:
                    ChangeToEraser();
                    EnablePointer();
                    break;
                case QvPen_Pen_State.PenUsing:
                    FinishDrawing();
                    ChangeToEraser();
                    EnablePointer();
                    break;
                case QvPen_Pen_State.EraserIdle:
                    EnablePointer();
                    break;
            }
            currentState = QvPen_Pen_State.EraserUsing;
        }

        #endregion

        public bool _TakeOwnership()
        {
            if (Networking.IsOwner(gameObject))
            {
                return true;
            }
            else
            {
                Networking.SetOwner(Networking.LocalPlayer, gameObject);
                return Networking.IsOwner(gameObject);
            }
        }

        [System.NonSerialized]
        public bool isPickedUp = false; // protected
        public bool isHeld => isPickedUp;

        public void _Respawn()
        {
            pickup.Drop();

            if (Networking.IsOwner(gameObject))
            {
                if (Utilities.IsValid(objectSync))
                    objectSync.Respawn();
                else if (Utilities.IsValid(_alternativeObjectSync))
                    _alternativeObjectSync.SendCustomEvent(_respawnEventName);
            }
        }

        public void _Clear()
        {
            manager.Clear(penId);
        }

        public void _EraseOwnInk()
        {
            _TakeOwnership();
            SendEraseOwnInk();
        }

        public void _UndoDraw()
        {
            _TakeOwnership();
            SendUndoDraw();
        }

        private void StartDrawing()
        {
            trailRenderer.gameObject.SetActive(true);
        }

        private void FinishDrawing()
        {
            if (isUser)
            {
                var inkId = penManager.InkId;

                var inkIdVector = QvPenUtilities.Int32ToVector3(inkId);
                var data = PackData(trailRenderer, QvPen_Pen_Mode.Draw, inkIdVector, localPlayerIdVector);

                AddLocalInkHistory(inkId);

                penManager._IncrementInkId();

                _SendData(data);
            }

            trailRenderer.gameObject.SetActive(false);
            trailRenderer.Clear();
        }

        private Vector3[] PackData(TrailRenderer trailRenderer, QvPen_Pen_Mode mode, Vector3 inkIdVector, Vector3 ownerIdVector)
        {
            if (!Utilities.IsValid(trailRenderer))
                return null;

            var positionCount = trailRenderer.positionCount;

            if (positionCount == 0)
                return null;

            var positions = new Vector3[positionCount];

            trailRenderer.GetPositions(positions);

            System.Array.Reverse(positions);

            var data = new Vector3[positionCount + GetFooterSize(mode)];

            System.Array.Copy(positions, data, positionCount);

            var modeAsInt = (int)mode; // Compiler bug

            SetData(data, FOOTER_ELEMENT_DATA_INFO, new Vector3(localPlayerId, modeAsInt, GetFooterSize(mode)));
            SetData(data, FOOTER_ELEMENT_PEN_ID, penIdVector);
            SetData(data, FOOTER_ELEMENT_INK_ID, inkIdVector);
            SetData(data, FOOTER_ELEMENT_OWNER_ID, ownerIdVector);
            SetData(data, FOOTER_ELEMENT_DRAW_INK_INFO, new Vector3(inkMeshLayer, inkColliderLayer, enabledLateSync ? 1f : 0f));

            return data;
        }

        public Vector3[] _PackData(LineRenderer lineRenderer, QvPen_Pen_Mode mode, Vector3 inkIdVector, Vector3 ownerIdVector)
        {
            if (!Utilities.IsValid(lineRenderer))
                return null;

            var positionCount = lineRenderer.positionCount;

            if (positionCount == 0)
                return null;

            var positions = new Vector3[positionCount];

            lineRenderer.GetPositions(positions);

            var data = new Vector3[positionCount + GetFooterSize(mode)];

            System.Array.Copy(positions, data, positionCount);

            var inkMeshLayer = lineRenderer.gameObject.layer;
            var inkColliderLayer = lineRenderer.GetComponentInChildren<MeshCollider>(true).gameObject.layer;

            var modeAsInt = (int)mode; // Compiler bug

            SetData(data, FOOTER_ELEMENT_DATA_INFO, new Vector3Int(localPlayerId, modeAsInt, GetFooterSize(mode)));
            SetData(data, FOOTER_ELEMENT_PEN_ID, penIdVector);
            SetData(data, FOOTER_ELEMENT_INK_ID, inkIdVector);
            SetData(data, FOOTER_ELEMENT_OWNER_ID, ownerIdVector);
            SetData(data, FOOTER_ELEMENT_DRAW_INK_INFO, new Vector3Int(inkMeshLayer, inkColliderLayer, enabledLateSync ? 1 : 0));

            return data;
        }

        public void _SendData(Vector3[] data) => penManager._SendData(data);

        private void EnablePointer()
        {
            isPointerEnabled = true;

            if (Utilities.IsValid(pointerRenderer))
                pointerRenderer.sharedMaterial = pointerMaterialActive;
        }

        private void DisablePointer()
        {
            isPointerEnabled = false;

            if (Utilities.IsValid(pointerRenderer))
                pointerRenderer.sharedMaterial = pointerMaterialNormal;
        }

        private void ChangeToPen()
        {
            DisablePointer();
            pointer.gameObject.SetActive(false);
        }

        private void ChangeToEraser()
        {
            pointer.gameObject.SetActive(true);
        }

        public void _UnpackData(Vector3[] data, QvPen_Pen_Mode targetMode)
        {
            var mode = GetMode(data);

            if (targetMode != QvPen_Pen_Mode.Any && mode != targetMode)
                return;

            switch (mode)
            {
                case QvPen_Pen_Mode.Draw:
                    CreateInkInstance(data);
                    break;
                case QvPen_Pen_Mode.Erase:
                    EraseInk(data);
                    break;
                case QvPen_Pen_Mode.EraseUserInk:
                    EraseUserInk(data);
                    break;
            }
        }

        public void _EraseAbandonedInk(Vector3[] data)
        {
            var mode = GetMode(data);

            if (mode != QvPen_Pen_Mode.Draw)
                return;

            EraseInk(data);
        }

        private void AddLocalInkHistory(int inkId)
        {
            if (localInkHistory.Count > 1024)
                localInkHistory.RemoveAt(0);

            localInkHistory.Add(inkId);
        }

        private bool TryGetLastLocalInk(out int inkId)
        {
            for (int i = localInkHistory.Count - 1; i >= 0; i--)
            {
                if (!localInkHistory.TryGetValue(i, TokenType.Int, out var inkIdToken))
                    continue;

                inkId = inkIdToken.Int;

                if (!manager.HasInk(penId, inkId))
                {
                    localInkHistory.RemoveAt(i);
                    continue;
                }

                return true;
            }

            inkId = default;
            return false;
        }

        #region Draw Line

        private void CreateInkInstance(Vector3[] data)
        {
            var penIdVector = GetData(data, FOOTER_ELEMENT_PEN_ID);
            var inkIdVector = GetData(data, FOOTER_ELEMENT_INK_ID);

            var penId = QvPenUtilities.Vector3ToInt32(penIdVector);
            var inkId = QvPenUtilities.Vector3ToInt32(inkIdVector);

            if (manager.HasInk(penId, inkId))
                return;

            var playerIdVector = GetData(data, FOOTER_ELEMENT_OWNER_ID);

            var lineInstance = Instantiate(inkPrefab.gameObject);
            lineInstance.name = $"{inkPrefix} ({inkId})";

            if (!QvPenUtilities.TrySetIdFromInk(lineInstance, penIdVector, inkIdVector, playerIdVector))
            {
                Warning($"Failed TrySetIdFromInk pen: {penId}, ink: {inkId}");
                Destroy(lineInstance);
                return;
            }

            manager.SetInk(penId, inkId, lineInstance);

            var inkInfo = GetData(data, FOOTER_ELEMENT_DRAW_INK_INFO);
            lineInstance.layer = (int)inkInfo.x;
            lineInstance.GetComponentInChildren<MeshCollider>(true).gameObject.layer = (int)inkInfo.y;
            QvPenUtilities.SetParentAndResetLocalTransform(
                lineInstance.transform, (int)inkInfo.z == 1 ? inkPoolSynced : inkPoolNotSynced);

            var positionCount = data.Length - GetFooterLength(data);

            var line = lineInstance.GetComponent<LineRenderer>();

            line.positionCount = positionCount;
            line.SetPositions(data);

#if UNITY_STANDALONE
            if (isRoundedTrailShader)
            {
                if (!Utilities.IsValid(propertyBlock))
                    propertyBlock = new MaterialPropertyBlock();
                else
                    propertyBlock.Clear();

                line.GetPropertyBlock(propertyBlock);
                propertyBlock.SetFloat("_Width", inkWidth);
                line.SetPropertyBlock(propertyBlock);
            }
            else
            {
                line.widthMultiplier = inkWidth;
            }
#endif

            CreateInkCollider(line);

            lineInstance.SetActive(true);
        }

        private void CreateInkCollider(LineRenderer lineRenderer)
        {
            var inkCollider = lineRenderer.GetComponentInChildren<MeshCollider>(true);
            inkCollider.name = "InkCollider";

            var mesh = new Mesh();

            {
                var tmpWidthMultiplier = lineRenderer.widthMultiplier;

                lineRenderer.widthMultiplier = inkWidth;
                lineRenderer.BakeMesh(mesh);
                lineRenderer.widthMultiplier = tmpWidthMultiplier;
            }

            inkCollider.GetComponent<MeshCollider>().sharedMesh = mesh;
            inkCollider.gameObject.SetActive(true);
        }

        #endregion

        #region Erase Line

        private void SendEraseInk(Vector3 penIdVector, Vector3 inkIdVector)
        {
            var data = new Vector3[GetFooterSize(QvPen_Pen_Mode.Erase)];

            SetData(data, FOOTER_ELEMENT_DATA_INFO,
                new Vector3(localPlayerId, (int)QvPen_Pen_Mode.Erase, GetFooterSize(QvPen_Pen_Mode.Erase)));
            SetData(data, FOOTER_ELEMENT_PEN_ID, penIdVector);
            SetData(data, FOOTER_ELEMENT_INK_ID, inkIdVector);

            _SendData(data);
        }

        private void SendEraseInk(int penId, int inkId)
        {
            SendEraseInk(QvPenUtilities.Int32ToVector3(penId), QvPenUtilities.Int32ToVector3(inkId));
        }

        private void SendEraseInk(GameObject ink)
        {
            if (Utilities.IsValid(ink)
             && QvPenUtilities.TryGetIdFromInk(ink, out var penIdVector, out var inkIdVector, out var _discard))
            {
                SendEraseInk(penIdVector, inkIdVector);
            }
        }

        private void SendEraseOwnInk()
        {
            var data = new Vector3[GetFooterSize(QvPen_Pen_Mode.EraseUserInk)];

            SetData(data, FOOTER_ELEMENT_DATA_INFO,
                new Vector3(localPlayerId, (int)QvPen_Pen_Mode.EraseUserInk, GetFooterSize(QvPen_Pen_Mode.EraseUserInk)));
            SetData(data, FOOTER_ELEMENT_PEN_ID, penIdVector);
            SetData(data, FOOTER_ELEMENT_OWNER_ID, localPlayerIdVector);

            _SendData(data);
        }

        private void SendUndoDraw()
        {
            if (!TryGetLastLocalInk(out var inkId))
                return;

            SendEraseInk(penId, inkId);
        }

        private void EraseInk(Vector3[] data)
        {
            if (data.Length < GetFooterSize(QvPen_Pen_Mode.Erase))
                return;

            var penIdVector = GetData(data, FOOTER_ELEMENT_PEN_ID);
            var inkIdVector = GetData(data, FOOTER_ELEMENT_INK_ID);

            var penId = QvPenUtilities.Vector3ToInt32(penIdVector);
            var inkId = QvPenUtilities.Vector3ToInt32(inkIdVector);

            manager.RemoveInk(penId, inkId);
        }

        private void EraseUserInk(Vector3[] data)
        {
            if (data.Length < GetFooterSize(QvPen_Pen_Mode.EraseUserInk))
                return;

            var ownerIdVector = GetData(data, FOOTER_ELEMENT_OWNER_ID);

            var penId = QvPenUtilities.Vector3ToInt32(penIdVector);

            manager.RemoveUserInk(penId, ownerIdVector);
        }

        #endregion

        #region Tool

        private const string UDON_EVENT_INTERACT = "_interact";

        private readonly Collider[] results32 = new Collider[32];
        private void InteractOtherUdon()
        {
            var count = Physics.OverlapSphereNonAlloc(pointer.position, pointerRadius, results32, Physics.AllLayers, QueryTriggerInteraction.Collide);
            for (var i = 0; i < count; i++)
            {
                var other = results32[i];

                if (Utilities.IsValid(other))
                {
                    var udonComponents = other.GetComponents(typeof(VRC.Udon.UdonBehaviour));

                    foreach (var udonComponent in udonComponents)
                    {
                        if (!Utilities.IsValid(udonComponent))
                            continue;

                        var udon = (VRC.Udon.UdonBehaviour)udonComponent;

                        if (udon.DisableInteractive)
                            continue;

                        udon.SendCustomEvent(UDON_EVENT_INTERACT);
                    }
                }

                results32[i] = null;
            }

            System.Array.Clear(results32, 0, results32.Length);
        }

        #endregion

        #region Log

        private const string appName = nameof(QvPen_Pen);

        private void Log(object o) => Debug.Log($"{logPrefix}{o}", this);
        private void Warning(object o) => Debug.LogWarning($"{logPrefix}{o}", this);
        private void Error(object o) => Debug.LogError($"{logPrefix}{o}", this);

        private readonly Color logColor = new Color(0xf2, 0x7d, 0x4a, 0xff) / 0xff;
        private string ColorBeginTag(Color c) => $"<color=\"#{ToHtmlStringRGB(c)}\">";
        private const string ColorEndTag = "</color>";

        private string _logPrefix;
        private string logPrefix
            => !string.IsNullOrEmpty(_logPrefix)
                ? _logPrefix : (_logPrefix = $"[{ColorBeginTag(logColor)}{nameof(QvPen)}.{nameof(QvPen.Udon)}.{appName}{ColorEndTag}] ");

        private static string ToHtmlStringRGB(Color c)
        {
            c *= 0xff;
            return $"{Mathf.RoundToInt(c.r):x2}{Mathf.RoundToInt(c.g):x2}{Mathf.RoundToInt(c.b):x2}";
        }

        #endregion
    }

    #region QvPenUtilities

    public static class QvPenUtilities
    {
        public static void SetParentAndResetLocalTransform(Transform child, Transform parent)
        {
            if (!Utilities.IsValid(child))
                return;

            child.SetParent(parent);
            child.SetLocalPositionAndRotation(Vector3.zero, Quaternion.identity);
            child.localScale = Vector3.one;
        }

        public static Vector3 Int32ToVector3(int v)
            => new Vector3((v >> 24) & 0x00ff, (v >> 12) & 0x0fff, v & 0x0fff);

        public static int Vector3ToInt32(Vector3 v)
            => ((int)v.x & 0x00ff) << 24 | ((int)v.y & 0x0fff) << 12 | ((int)v.z & 0x0fff);

        const int PIDB = 360;
        const float PIDD = PIDB / 90f;

        public static Vector3 GetPlayerIdVector(int playerId)
        {
            var x = playerId;
            var y = x / PIDB;
            var z = y / PIDB;
            return new Vector3(x % PIDB, y % PIDB, z % PIDB) / PIDD;
        }

        public static int EulerAnglesToPlayerId(Vector3 v)
        {
            v *= PIDD;
            return Mathf.RoundToInt(v.x)
                + Mathf.RoundToInt(v.y) * PIDB
                + Mathf.RoundToInt(v.z) * (PIDB * PIDB);
        }

        public static bool TryGetIdFromInk(GameObject ink,
            out Vector3 penIdVector, out Vector3 inkIdVector, out Vector3 ownerIdVector)
        {
            if (!Utilities.IsValid(ink))
            {
                penIdVector = default;
                inkIdVector = default;
                ownerIdVector = default;
                return false;
            }

            if (ink.transform.childCount < 2)
            {
                penIdVector = default;
                inkIdVector = default;
                ownerIdVector = default;
                return false;
            }

            var idHolder = ink.transform.GetChild(1);
            if (!Utilities.IsValid(idHolder))
            {
                penIdVector = default;
                inkIdVector = default;
                ownerIdVector = default;
                return false;
            }

            penIdVector = idHolder.localPosition;
            inkIdVector = idHolder.localScale;
            ownerIdVector = idHolder.localEulerAngles;
            return true;
        }

        public static bool TrySetIdFromInk(GameObject ink,
             Vector3 penIdVector, Vector3 inkIdVector, Vector3 ownerIdVector)
        {
            if (!Utilities.IsValid(ink))
                return false;

            if (ink.transform.childCount < 2)
                return false;

            var idHolder = ink.transform.GetChild(1);
            if (!Utilities.IsValid(idHolder))
                return false;

            idHolder.localPosition = penIdVector;
            idHolder.localScale = inkIdVector;
            idHolder.localEulerAngles = ownerIdVector;
            return true;
        }
    }

    #endregion

    #region Enum

    public enum QvPen_Pen_SyncState
    {
        Idle,
        Started,
        Finished
    }

    enum QvPen_Pen_State
    {
        PenIdle,
        PenUsing,
        EraserIdle,
        EraserUsing
    }

    public enum QvPen_Pen_Mode
    {
        None,
        Any,
        Draw,
        Erase,
        EraseUserInk
    }

    static class QvPen_Pen_Extension
    {
        internal static string ToStr(this QvPen_Pen_State state)
        {
            switch (state)
            {
                case QvPen_Pen_State.PenIdle: return nameof(QvPen_Pen_State.PenIdle);
                case QvPen_Pen_State.PenUsing: return nameof(QvPen_Pen_State.PenUsing);
                case QvPen_Pen_State.EraserIdle: return nameof(QvPen_Pen_State.EraserIdle);
                case QvPen_Pen_State.EraserUsing: return nameof(QvPen_Pen_State.EraserUsing);
                default: return "(QvPen_Pen_State.???)";
            }
        }
    }

    #endregion
}
```

```json
{
  "byteCodeHex": "000000010000004D000000010000000B000000010000004C000000090000000100000002000000090000000800000002000000010000004D000000010000000F00000004000000780000000100000010000000010000004E0000000900000001000000020000000900000008000000020000000500000234000000010000000E000000010000020400000006000003EF0000000100000204000000010000004F000000010000020300000006000003F00000000100000203000000010000005000000006000003F1000000010000000E000000010000020500000006000003F20000000100000205000000010000020600000006000003F300000001000000520000000100000053000000010000020600000006000003F40000000100000205000000010000020700000006000003F500000001000000520000000100000054000000010000020700000006000003F40000000100000205000000010000020800000006000003F600000001000000520000000100000055000000010000020800000006000003F40000000100000052000000010000020900000006000003F700000001000000510000000100000209000000010000020A00000006000003F80000000100000203000000010000020B00000006000003F9000000010000020A000000010000020B000000010000001000000006000003FA0000000100000056000000010000000F000000090000000100000010000000010000004E0000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000004D0000000100000058000000050000079C0000000100000059000000040000028C000000010000005A00000001000000570000000900000005000002A40000000100000011000000010000005700000006000003FB0000000100000002000000090000000800000002000000010000004D0000000100000018000000010000020C00000006000003FC000000010000020C00000004000003040000000100000018000000010000005B0000000900000005000003580000000100000004000000010000020D00000006000003EF000000010000020D0000000100000056000000010000005C000000010000001800000006000003FD0000000100000018000000010000005B000000090000000100000002000000090000000800000002000000010000004D0000000100000019000000010000005D000000090000000100000002000000090000000800000002000000010000004D000000010000001A000000010000020E00000006000003FC000000010000020E00000004000003E8000000010000001A000000010000005E000000090000000500000430000000010000005F0000000100000060000000010000020F00000006000003FE000000010000020F000000010000001A00000009000000010000001A000000010000005E000000090000000100000002000000090000000800000002000000010000004D000000010000001B000000010000021000000006000003FC00000001000002100000000400000490000000010000001B00000001000000610000000900000005000004D800000001000000620000000100000063000000010000021100000006000003FE0000000100000211000000010000001B00000009000000010000001B0000000100000061000000090000000100000002000000090000000800000002000000010000004D000000010000002B00000001000000640000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000004D0000000100000065000000010000002B000000090000000100000002000000090000000800000002000000010000004D000000010000003000000001000000660000000900000001000000660000000100000067000000010000021200000006000003FF000000010000021200000004000005D00000000100000030000000060000040000000001000000300000000100000066000000090000000100000002000000090000000800000002000000010000004D00000001000000310000000400000618000000010000003200000001000000680000000900000005000006A8000000010000006900000005000005680000000100000066000000010000003100000006000003FC00000001000000310000000400000694000000010000006A0000000500000568000000010000006600000001000000320000000600000401000000010000003200000001000000680000000900000005000006A800000001000000530000000100000068000000090000000100000002000000090000000800000002000000010000004D000000010000003300000004000006FC0000000100000034000000010000006B000000090000000100000002000000090000000800000002000000010000006C000000010000006F00000005000005EC0000000100000068000000010000006E000000090000000500006F68000000010000006D00000001000000340000000900000001000000560000000100000033000000090000000100000034000000010000006B0000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000004D000000010000003500000004000007C800000001000000360000000100000059000000090000000500000864000000010000007000000005000005680000000100000066000000010000003500000006000003FC00000001000000350000000100000213000000090000000100000213000000040000085000000001000000710000000500000568000000010000006600000001000000360000000600000402000000010000003600000001000002130000000900000001000002130000000100000059000000090000000100000002000000090000000800000002000000010000004D0000000100000072000000010000001E000000090000000100000073000000050000110000000001000000740000000100000075000000010000021500000006000004030000000100000215000000010000021400000006000004040000000100000214000000010000021600000006000003FC0000000100000216000000040000095400000001000000070000000100000232000000060000040500000001000002320000000100000050000000060000040600000001000002140000000100000007000000060000040700000005000009F8000000010000000700000001000000750000000600000408000000010000007600000001000000670000000100000229000000090000000100000007000000010000007700000009000000010000022900000001000000780000000900000005000070D400000001000000070000000600000409000000010000000700000001000002320000000600000405000000010000023200000001000000560000000600000406000000010000000D000000010000000800000006000003EF00000001000000790000000100000008000000010000007700000009000000010000000700000001000000780000000900000005000070D4000000010000007A0000000100000217000000060000040A00000001000002170000000100000218000000060000040B00000001000002180000000400000AA40000000100000053000000010000002A000000090000000500000ABC0000000100000217000000010000002A000000060000040C000000010000007B000000010000007C000000010000002A000000010000007E00000009000000050000717C000000010000007D0000000100000065000000090000000500000538000000010000008000000005000004F40000000100000064000000010000021900000006000003F30000000100000219000000010000021B000000060000040D000000010000021B000000010000021C000000060000040E000000010000021C000000010000021A000000060000040F000000010000008100000005000004F40000000100000064000000010000021D00000006000003F5000000010000021D000000010000021F000000060000040D000000010000021F0000000100000220000000060000040E0000000100000220000000010000021E000000060000040F000000010000008200000005000004F40000000100000064000000010000022100000006000003F600000001000002210000000100000223000000060000040D00000001000002230000000100000224000000060000040E00000001000002240000000100000222000000060000040F000000010000007F000000010000021A000000010000021E0000000100000222000000010000002C000000060000041000000001000000830000000100000084000000010000002C000000010000022500000006000004110000000100000008000000010000022500000006000004080000000100000072000000010000007200000001000002260000000900000001000000850000000600000412000000010000007200000001000000860000000100000228000000060000041300000001000002280000000100000227000000090000000100000227000000010000000B0000000900000001000000870000000100000007000000010000022900000006000003EF000000010000022900000001000000890000000900000005000072A00000000100000088000000010000000C00000009000000010000002A000000010000022A00000009000000010000000C000000010000008B000000010000002A0000000600000414000000010000000C000000010000008C000000010000008A0000000600000414000000010000000C000000010000008D0000000600000412000000010000000D000000010000008C000000010000008E0000000600000414000000010000000D000000010000008F0000000600000412000000010000000D000000010000000D000000010000022B0000000900000001000000900000000600000412000000010000000D0000000100000091000000010000022D0000000600000413000000010000022D000000010000022C00000009000000010000022C000000010000000900000009000000010000000D000000010000000D000000010000022E0000000900000001000000920000000600000412000000010000000D0000000100000093000000010000023000000006000004130000000100000230000000010000022F00000009000000010000022F000000010000000A00000009000000010000009400000005000003A4000000010000005E00000001000000950000000600000415000000010000009600000005000003A4000000010000005E00000001000000970000000600000416000000010000000E000000010000023100000006000003EF00000001000002310000000100000098000000010000002300000006000003F0000000010000000E00000001000002320000000600000405000000010000023200000001000000500000000600000406000000010000000E000000010000023300000006000003EF0000000100000233000000010000023400000006000004170000000100000099000000050000025000000001000002340000000100000057000000010000023500000006000004180000000100000233000000010000023500000006000004190000000100000015000000010000023600000006000003EF000000010000009A000000010000002D00000001000002370000000600000418000000010000023600000001000002370000000600000419000000010000009B000000050000079C000000010000005900000004000010D0000000010000009C00000001000000260000000900000005000010E4000000010000009D0000000100000026000000090000000100000002000000090000000800000002000000010000004D000000010000001E000000010000009E000000010000023800000006000004130000000100000238000000010000002D000000060000041A000000010000001E000000010000009F000000010000023900000006000004130000000100000239000000010000001F000000060000041B000000010000001E00000001000000A0000000010000023A0000000600000413000000010000023A0000000100000020000000060000041B000000010000005400000001000000200000000100000021000000060000041C0000000100000004000000010000023B0000000600000405000000010000023B000000010000001F000000060000041D0000000100000003000000010000023C0000000600000405000000010000023C000000010000001F000000060000041D00000001000000A100000005000002C0000000010000005B000000010000023D0000000600000405000000010000023D0000000100000020000000060000041D000000010000001E00000001000000A2000000010000023F0000000600000413000000010000023F000000010000023E000000090000000100000004000000010000023E000000060000041E0000000100000003000000010000023E000000060000041E000000010000023E000000010000024000000006000003FC00000001000002400000000400001434000000010000023E0000000100000247000000060000041F0000000100000247000000010000024800000006000003FC000000010000024800000004000014340000000100000247000000010000024900000009000000010000001E000000010000001E000000010000024A0000000900000001000000A30000000600000412000000010000001E00000001000000A4000000010000024100000006000004130000000100000241000000010000024B00000009000000010000024B000000010000024C000000090000000100000249000000010000024C000000010000002E00000006000004200000000100000247000000010000024D0000000600000421000000010000024D00000001000000A5000000010000024E0000000600000422000000010000002E000000010000024E000000010000002E0000000600000423000000010000002E000000040000153C000000010000000400000001000000A60000000600000424000000010000002F00000006000004250000000100000004000000010000002F0000000600000426000000010000002F00000001000000A7000000010000002D00000006000004270000000100000004000000010000002F0000000600000428000000010000000300000001000000A60000000600000429000000010000002F000000060000042A0000000100000003000000010000002F0000000600000426000000010000002F00000001000000A7000000010000002D00000006000004270000000100000003000000010000002F0000000600000428000000050000156C0000000100000004000000010000002D00000006000004240000000100000003000000010000002D0000000600000429000000010000001E00000001000000A800000001000002410000000600000413000000010000024100000001000002420000000900000001000000040000000100000242000000060000042B000000010000001E00000001000000A800000001000002430000000600000413000000010000024300000001000002440000000900000001000000030000000100000244000000060000042C000000010000001E00000001000000A900000001000002450000000600000413000000010000024500000001000002460000000900000001000002460000000100000046000000060000042D0000000100000002000000090000000800000002000000010000004D00000001000000AC00000001000000AB000000010000024F0000000900000005000004F4000000010000024F000000010000006400000001000000AA000000060000042E0000000100000002000000090000000800000002000000010000004D00000001000000AD000000050000079C000000010000005900000001000002500000000900000001000002500000000400001708000000050000172000000001000000190000000100000250000000060000042F00000001000002500000000400001744000000010000000200000009000000080000000200000001000000AE000000010000025100000006000004300000000100000251000000040000177C00000001000000AF0000000500001BF800000001000002520000000600000431000000010000025200000004000017A400000005000017B8000000010000000200000009000000080000000200000001000000B000000001000002530000000600000432000000010000025300000004000017F800000001000000B100000005000040540000000500001A6000000001000000AE000000010000025400000006000004320000000100000254000000040000183800000001000000B20000000500001A7C0000000500001A6000000001000000AE0000000100000255000000060000043300000001000002550000000400001A6000000001000000B300000001000002560000000600000432000000010000025600000004000018B0000000010000001E00000001000000B400000001000000B500000006000004340000000500001A6000000001000000B60000000100000257000000060000043300000001000002570000000400001900000000010000001E00000001000000B400000001000000B700000006000004340000000500001A6000000001000000B800000001000002580000000600000433000000010000025800000004000019B40000000100000042000000010000009D000000010000025A0000000600000435000000010000025A00000001000000B90000000100000042000000060000043600000001000000BA00000001000000BC0000000100000042000000010000025B0000000600000403000000010000025B00000001000000BB000000090000000500006C200000000500001A6000000001000000BD0000000100000259000000060000043300000001000002590000000400001A600000000100000042000000010000009D000000010000025A0000000600000437000000010000025A0000000100000051000000010000004200000006000003F800000001000000BE00000001000000BC0000000100000042000000010000025B0000000600000403000000010000025B00000001000000BB000000090000000500006C200000000100000002000000090000000800000002000000010000004D000000010000005600000001000000430000000900000001000000150000000100000056000000060000043800000001000000BF000000010000003C000000090000000100000014000000010000025C0000000600000405000000010000025C000000010000005600000006000004060000000100000014000000010000025D00000006000003EF000000010000025D00000001000000C0000000010000025E00000006000003F0000000010000025E000000010000025F0000000600000439000000010000025F000000010000003D000000060000043A000000010000001400000001000002600000000600000405000000010000026000000001000000500000000600000406000000010000003D00000001000000C1000000010000003E000000060000043B000000010000003D0000000100000261000000060000043C00000001000000C200000001000002610000000100000040000000060000043D0000000100000002000000090000000800000002000000010000004D000000010000005000000001000000430000000900000001000000C3000000050000292800000001000000C40000000400001C340000000500001C4C00000001000000150000000100000050000000060000043800000001000000C500000001000000B400000001000000C60000000600000434000000010000000600000001000000C700000001000000C8000000060000043E0000000100000003000000010000026200000006000003EF00000001000000060000000100000263000000060000043F00000001000000060000000100000264000000060000044000000001000002620000000100000263000000010000026400000006000004410000000100000002000000090000000800000002000000010000004D00000001000000C90000000500003E6800000001000000CA0000000400001D380000000500001D4C000000010000000200000009000000080000000200000001000000CB000000050000079C00000001000000590000000100000266000000060000042F00000001000002660000000400001D980000000100000019000000010000026600000009000000010000026600000001000002650000000900000001000002650000000400001DD400000001000000AE000000010000026500000006000004330000000100000265000000040000216400000001000000CC0000000500000568000000010000006600000001000000CD000000010000003800000006000004420000000100000038000000010000003900000006000004430000000100000038000000010000003B0000000600000444000000010000003B00000001000000CE00000001000002670000000600000445000000010000003B00000001000000CE0000000100000268000000060000044500000001000000CF0000000100000269000000060000043F00000001000002690000000100000039000000010000026A00000006000004460000000100000268000000010000026A000000010000026B00000006000004470000000100000267000000010000026B000000010000003A0000000600000418000000010000003B00000001000000CE000000010000026C0000000600000445000000010000026C000000010000003A000000010000026D00000006000004470000000100000040000000010000026D000000010000004100000006000003FA000000010000003A0000000100000039000000010000003A000000060000044800000001000000D0000000010000026E0000000600000449000000010000003F000000010000026E000000060000044A000000010000003F00000001000002700000000900000001000000D1000000010000026F0000000600000449000000010000003F000000010000026F000000060000044B000000010000003F000000010000027100000009000000010000026E000000060000044C0000000100000042000000010000026E000000010000026F00000006000003FA000000010000026F000000010000003F0000000100000270000000060000044D000000010000003C0000000100000270000000010000003C000000060000044E000000010000003E0000000100000271000000060000044F000000010000003C0000000100000271000000010000027200000006000004500000000100000272000000010000003E000000010000003C0000000600000451000000010000003C00000001000002730000000600000452000000010000003B0000000100000273000000010000027400000006000004450000000100000274000000010000004100000001000002750000000600000418000000010000003A00000001000002750000000100000276000000060000044800000001000000060000000100000276000000010000003B000000060000044100000001000000D2000000050000292800000001000000C400000004000022AC000000010000004300000004000021B400000001000000060000000100000277000000060000043F00000005000021CC00000001000000050000000100000277000000060000043F00000001000000470000000100000277000000010000027800000006000004530000000100000278000000010000027700000001000002790000000600000454000000010000002D00000001000000D3000000010000026B000000060000043D00000001000002780000000100000277000000010000026B000000010000026700000006000004550000000100000006000000010000026700000006000004560000000100000279000000010000005A000000010000027A0000000600000457000000010000027A00000004000022AC00000001000000D40000000500002CC0000000010000002200000004000022C4000000050000248C00000001000000190000000400002424000000010000026B000000060000044C000000010000026B00000001000000D5000000010000027B00000006000003FA0000000100000003000000010000027C00000006000003EF0000000100000003000000010000027D00000006000003EF000000010000027D0000000100000267000000060000043F00000001000000060000000100000268000000060000043F00000001000002670000000100000268000000010000027B000000010000026900000006000004580000000100000003000000010000027E00000006000003EF000000010000027E000000010000027F0000000600000440000000010000000600000001000002800000000600000440000000010000027F0000000100000280000000010000027B00000001000002810000000600000459000000010000027C000000010000026900000001000002810000000600000441000000050000248C0000000100000003000000010000027C00000006000003EF00000001000000060000000100000267000000060000043F0000000100000006000000010000027F0000000600000440000000010000027C0000000100000267000000010000027F00000006000004410000000100000002000000090000000800000002000000010000004D000000010000001900000004000024C000000005000024D400000001000000020000000900000008000000020000000100000022000000040000290C000000010000000E0000000100000283000000060000043F00000001000000D6000000010000000E00000001000002840000000900000005000000380000000100000283000000010000004E0000000100000044000000010000002100000001000000D70000000100000282000000060000045A0000000100000053000000010000028500000009000000010000028500000001000002820000000100000286000000060000045B0000000100000286000000040000290C000000010000004400000001000002850000000100000287000000060000045C0000000100000287000000010000028C00000006000003FC000000010000028C000000040000262C0000000100000287000000010000028D00000006000003EF000000010000028D0000000100000288000000060000045D0000000100000288000000010000028C00000006000003FC000000010000028C000000010000028B00000009000000010000028B000000040000268000000001000002880000000100000289000000060000045D0000000100000289000000010000028B00000006000003FC000000010000028B00000004000028B00000000100000016000000040000275C0000000100000289000000010000028A000000060000045D000000010000028A000000010000029000000006000003FC00000001000002900000000400002740000000010000028A0000000100000291000000060000045D0000000100000291000000010000029200000009000000010000000700000001000002930000000900000001000002920000000100000293000000010000029000000006000004200000000100000290000000010000028F0000000900000005000027BC00000001000002890000000100000294000000060000045D0000000100000294000000010000029500000009000000010000000800000001000002960000000900000001000002950000000100000296000000010000028F0000000600000420000000010000028F00000004000028B00000000100000287000000010000029800000006000003EF000000010000029800000001000000D80000000100000297000000060000045E0000000100000297000000010000029900000006000003FC000000010000029900000004000028640000000100000297000000010000029A000000060000045F000000010000029A000000010000005300000001000002990000000600000460000000010000029900000004000028B000000001000000D90000000100000297000000010000029B0000000600000405000000010000029B00000001000000DA0000000900000005000061040000000100000067000000010000028E0000000900000001000000440000000100000285000000010000028E00000006000004610000000100000285000000010000005400000001000002850000000600000462000000050000256C0000000100000002000000090000000800000002000000010000004D000000010000004700000001000000C400000006000004630000000100000002000000090000000800000002000000010000004D0000000100000019000000010000029E00000009000000010000029E00000004000029940000000100000045000000010000029E00000009000000010000029E000000010000029D00000009000000010000029D00000004000029D000000001000000DB000000010000029D00000006000003FC000000010000029D000000010000029C00000009000000010000029C0000000400002A2400000001000000DB000000010000029F0000000600000464000000010000029F000000010000029C000000060000042F000000010000029C0000000400002C5C00000001000000DB00000001000002A0000000060000040500000001000002A000000001000002A10000000600000465000000010000005400000001000002A100000001000002A2000000060000041C00000001000002A2000000010000004600000001000002A3000000060000046600000001000002A3000000010000005300000001000002A4000000060000046700000001000002A40000000400002AE8000000010000000200000009000000080000000200000001000000DB00000001000002A6000000060000046800000001000002A6000000010000005C00000001000002A5000000060000046900000001000002A50000000400002B7400000001000000DB00000001000002A70000000900000001000002A700000001000002A8000000060000046A00000001000002A800000001000002A5000000060000042F00000001000002A50000000400002B980000000100000002000000090000000800000002000000010000000500000001000002AA000000060000043F00000001000000DB00000001000002AA00000001000002AB0000000600000453000000010000000500000001000002AC000000060000043F00000001000002AB00000001000002AC00000001000002A9000000060000045400000001000002A900000001000000DC00000001000002AD000000060000046B00000001000002AD0000000400002C5C00000001000000DD00000001000000DB00000001000000DE000000090000000500002C780000000100000002000000090000000800000002000000010000004D00000001000000DE0000000100000047000000090000000100000015000000010000005600000006000004380000000100000002000000090000000800000002000000010000004D000000010000006700000001000000470000000900000001000000430000000400002CEC0000000500002D0400000001000000150000000100000050000000060000043800000001000000DF00000001000000B400000001000000C60000000600000434000000010000000600000001000000C700000001000000C8000000060000043E000000010000000300000001000002AE00000006000003EF000000010000000600000001000002AF000000060000043F000000010000000600000001000002B0000000060000044000000001000002AE00000001000002AF00000001000002B000000006000004410000000100000002000000090000000800000002000000010000004D0000000100000056000000010000001900000009000000010000000C00000001000000E100000001000000E00000000600000414000000010000000C00000001000000E20000000600000412000000010000001E000000010000001E00000001000002B10000000900000001000000E30000000600000412000000010000001E000000010000001E00000001000002B20000000900000001000000E40000000600000412000000010000001E00000001000000E500000001000002B4000000060000041300000001000002B400000001000002B300000009000000010000001E00000001000000B400000001000000E6000000060000043400000001000000E700000001000000B400000001000000C600000006000004340000000100000002000000090000000800000002000000010000004D0000000100000050000000010000001900000009000000010000001E000000010000001E00000001000002B50000000900000001000000E80000000600000412000000010000001E00000001000000B400000001000000E9000000060000043400000001000000EA00000001000000B400000001000000C60000000600000434000000010000001E000000010000001E00000001000002B60000000900000001000000EB000000060000041200000001000000EC0000000500001BF800000001000000ED0000000500002CC00000000100000002000000090000000800000002000000010000004D000000010000002400000001000002B80000000900000001000002B8000000040000305800000001000002B9000000060000046C00000001000002B9000000010000002500000001000002BA000000060000043700000001000002BA00000001000000EE00000001000002B8000000060000046B00000001000002B800000001000002B70000000900000001000002B700000004000030D4000000010000000500000001000002BB000000060000043F00000001000002BB000000010000002700000001000002BC000000060000045400000001000002BC000000010000002600000001000002B7000000060000046B00000001000002B700000004000032B000000001000000A6000000010000002500000009000000010000002800000001000000EF00000001000002BD000000060000046700000001000002BD00000004000031C8000000010000000500000001000002BE000000060000043F00000001000002BE000000010000002700000001000002BF000000060000045400000001000002BF00000001000000A600000001000002C0000000060000045700000001000002C000000004000031A000000001000000F0000000050000405400000001000000F100000001000000B400000001000000F2000000060000043400000005000032A8000000010000002800000001000000F300000001000002C1000000060000046700000001000002C100000004000032A000000001000000F400000001000000B400000001000000C6000000060000043400000005000032A800000005000032A000000001000000F500000001000000F8000000010000002800000001000000FA00000009000000050000750800000001000000F700000001000000F900000001000000FB00000001000002C2000000060000041100000001000002C200000001000000F6000000090000000500006D2000000005000032A8000000050000322800000005000034200000000100000025000000060000046C00000001000000050000000100000027000000060000043F000000010000002800000001000000EF00000001000002BD000000060000046700000001000002BD000000040000333000000001000000FC00000001000000B400000001000000FD00000006000004340000000500003420000000010000002800000001000000F300000001000002C0000000060000046700000001000002C0000000040000341800000001000000FE00000001000000B400000001000000FF0000000600000434000000010000010000000005000069180000000500003420000000050000341800000001000001010000000100000103000000010000002800000001000000FA000000090000000500007508000000010000010200000001000000F900000001000000FB00000001000002C2000000060000041100000001000002C200000001000000F6000000090000000500006D20000000050000342000000005000033A00000000100000002000000090000000800000002000000010000004D0000000100000028000000010000005300000001000002C3000000060000046D00000001000002C300000004000036040000000100000028000000010000010400000001000002C4000000060000046E00000001000002C400000004000036040000000100000105000000010000002800000001000002C5000000060000046F00000008000002C5000000010000010600000001000000B400000001000000C60000000600000434000000050000367C000000010000010700000001000000B400000001000000F20000000600000434000000050000367C0000000100000108000000010000010B000000010000002800000001000000FA0000000900000005000075080000000100000109000000010000010A00000001000000F900000001000002C6000000060000041100000001000002C600000001000000BB000000090000000500006C20000000050000367C000000010000010C000000010000010E000000010000002800000001000000FA0000000900000005000075080000000100000109000000010000010D00000001000000F900000001000002C7000000060000041100000001000002C700000001000000BB000000090000000500006C20000000050000367C000000010000010F0000000100000110000000010000002800000001000000FA000000090000000500007508000000010000010200000001000000F9000000010000011100000001000002C8000000060000041100000001000002C800000001000000F6000000090000000500006D20000000050000367C0000000100000002000000090000000800000002000000010000004D0000000100000112000000010000002400000009000000010000001900000004000036DC000000010000011300000001000000B400000001000000C600000006000004340000000100000002000000090000000800000002000000010000004D00000001000001140000000100000017000000090000000100000002000000090000000800000002000000010000004D00000001000001150000000100000045000000090000000100000019000000040000376C000000010000011600000001000000B400000001000000C600000006000004340000000100000002000000090000000800000002000000010000004D000000010000000800000001000002C900000006000003FC00000001000002C900000004000037E0000000010000000800000001000002CA000000060000040500000001000002CA000000010000005600000006000004060000000100000002000000090000000800000002000000010000004D000000010000000800000001000002CB00000006000003FC00000001000002CB0000000400003854000000010000000800000001000002CC000000060000040500000001000002CC000000010000005000000006000004060000000100000002000000090000000800000002000000010000004D00000001000001170000000500003FB0000000010000000800000001000002CD00000006000003FC00000001000002CD00000004000038E4000000010000000800000001000002CE000000060000040500000001000002CE00000001000002CF0000000900000001000002CF00000006000004700000000100000002000000090000000800000002000000010000004D0000000100000028000000010000011800000001000002D0000000060000046700000001000002D00000000400003948000000010000011900000005000040DC00000005000039E8000000010000002800000001000000F300000001000002D1000000060000046700000001000002D10000000400003990000000010000011A0000000500004E0000000005000039E80000000100000028000000010000011B00000001000002D2000000060000046700000001000002D200000004000039E8000000010000011C0000000500004D90000000010000011D0000000500004E0000000005000039E800000001000000EF0000000100000028000000090000000100000002000000090000000800000002000000010000004D000000010000002800000001000000EF00000001000002D3000000060000046700000001000002D30000000400003A60000000010000011E00000005000040900000000500003B20000000010000002800000001000000F300000001000002D4000000060000046700000001000002D40000000400003AB8000000010000011F0000000500004E00000000010000012000000005000040900000000500003B200000000100000028000000010000011B00000001000002D5000000060000046700000001000002D50000000400003B2000000001000001210000000500004D9000000001000001220000000500004E00000000010000012300000005000040900000000500003B2000000001000001180000000100000028000000090000000100000002000000090000000800000002000000010000004D000000010000002800000001000000EF00000001000002D6000000060000046700000001000002D60000000400003B9800000001000001240000000500004E5C0000000500003C380000000100000028000000010000011800000001000002D7000000060000046700000001000002D70000000400003BF0000000010000012500000005000040DC00000001000001260000000500004E5C0000000500003C380000000100000028000000010000011B00000001000002D8000000060000046700000001000002D80000000400003C3800000001000001270000000500004D900000000500003C3800000001000000F30000000100000028000000090000000100000002000000090000000800000002000000010000004D000000010000002800000001000000EF00000001000002D9000000060000046700000001000002D90000000400003CC000000001000001280000000500004E5C00000001000001290000000500004D200000000500003D700000000100000028000000010000011800000001000002DA000000060000046700000001000002DA0000000400003D28000000010000012A00000005000040DC000000010000012B0000000500004E5C000000010000012C0000000500004D200000000500003D70000000010000002800000001000000F300000001000002DB000000060000046700000001000002DB0000000400003D70000000010000012D0000000500004D200000000500003D70000000010000011B0000000100000028000000090000000100000002000000090000000800000002000000010000004D000000010000007A00000001000002DC000000060000047100000001000002DC0000000400003DF80000000100000056000000010000012E0000000900000001000000020000000900000008000000020000000500003E4C00000001000002DD000000060000040000000001000002DD000000010000007A0000000600000472000000010000007A000000010000012E000000060000047100000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000004D000000010000004800000001000000CA000000090000000100000002000000090000000800000002000000010000004D000000010000012F00000005000003A4000000010000005E0000000600000473000000010000007A00000001000002DE000000060000047100000001000002DE0000000400003F940000000100000130000000050000044C000000010000006100000001000002DF00000006000003FC00000001000002DF0000000400003F400000000100000131000000050000044C000000010000006100000006000004740000000500003F94000000010000001C00000001000002E000000006000003FC00000001000002E00000000400003F94000000010000001C000000010000001D00000001000002E100000009000000010000001D00000006000004120000000100000002000000090000000800000002000000010000004D000000010000002A00000001000002E200000009000000010000000C0000000100000132000000010000002A0000000600000414000000010000000C000000010000013300000006000004120000000100000002000000090000000800000002000000010000004D00000001000001340000000500003DA0000000010000013500000005000062400000000100000002000000090000000800000002000000010000004D00000001000001360000000500003DA00000000100000137000000050000644C0000000100000002000000090000000800000002000000010000004D000000010000000300000001000002E3000000060000040500000001000002E3000000010000005600000006000004060000000100000002000000090000000800000002000000010000004D000000010000001900000004000042A8000000010000001E000000010000001E00000001000002E60000000900000001000001380000000600000412000000010000001E000000010000013900000001000002E8000000060000041300000001000002E800000001000002E7000000060000041B00000001000002E700000001000002E500000009000000010000013A00000001000002E5000000010000007E00000009000000050000717C000000010000007D00000001000002E900000009000000010000013B0000000100000142000000010000000300000001000002EB0000000900000005000006C400000001000002EB000000010000013D000000090000000100000141000000010000013E0000000900000001000002E9000000010000013F00000009000000010000006B0000000100000140000000090000000500004304000000010000013C00000001000002EA00000009000000010000014300000001000002E50000000100000144000000090000000500005140000000010000001E000000010000001E00000001000002EC0000000900000001000001450000000600000412000000010000014600000001000002EA0000000100000147000000090000000500004CB8000000010000000300000001000002E4000000060000040500000001000002E400000001000000500000000600000406000000010000000300000006000004750000000100000002000000090000000800000002000000010000004D000000010000013D00000001000002ED00000006000003FC00000001000002ED0000000400004334000000050000435C0000000100000067000000010000013C000000090000000100000002000000090000000800000002000000010000013D00000001000002EE000000060000047600000001000002EE000000010000005300000001000002EF000000060000046700000001000002EF00000004000043CC0000000100000067000000010000013C00000009000000010000000200000009000000080000000200000001000002EE00000001000002F00000000600000477000000010000013D00000001000002F000000001000002F1000000060000047800000001000002F000000006000004790000000100000148000000010000013E000000010000014A00000009000000050000766C00000001000002EE000000010000014900000001000002F3000000060000046200000001000002F300000001000002F2000000060000047700000001000002F000000001000002F200000001000002EE000000060000047A000000010000013E00000001000002F400000009000000010000014B000000010000014F00000005000005EC000000010000006800000001000002F5000000060000047B00000001000002F400000001000002F6000000060000047B0000000100000150000000010000013E000000010000014A00000009000000050000766C000000010000014900000001000002F7000000060000047B00000001000002F500000001000002F600000001000002F700000001000002F8000000060000047C00000001000002F2000000010000014C000000090000000100000053000000010000014D0000000900000001000002F8000000010000014E0000000900000005000077F80000000100000151000000010000015200000005000004F400000001000002F2000000010000014C000000090000000100000054000000010000014D000000090000000100000064000000010000014E0000000900000005000077F8000000010000015300000001000002F2000000010000014C000000090000000100000055000000010000014D00000009000000010000013F000000010000014E0000000900000005000077F8000000010000015400000001000002F2000000010000014C000000090000000100000104000000010000014D000000090000000100000140000000010000014E0000000900000005000077F80000000100000155000000010000001F00000001000002F9000000060000047B000000010000002000000001000002FA000000060000047B000000010000001700000004000046EC000000010000005A00000001000002FB00000009000000050000470000000001000000A600000001000002FB0000000900000001000002F900000001000002FA00000001000002FB00000001000002FC000000060000047C00000001000002F2000000010000014C000000090000000100000156000000010000014D0000000900000001000002FC000000010000014E0000000900000005000077F800000001000002F2000000010000013C0000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000004D000000010000015800000001000002FD00000006000003FC00000001000002FD00000004000047E0000000050000480800000001000000670000000100000157000000090000000100000002000000090000000800000002000000010000015800000001000002FE000000060000045F00000001000002FE000000010000005300000001000002FF000000060000046700000001000002FF00000004000048780000000100000067000000010000015700000009000000010000000200000009000000080000000200000001000002FE00000001000003000000000600000477000000010000015800000001000003000000000100000301000000060000047D000000010000015C0000000100000159000000010000014A00000009000000050000766C00000001000002FE0000000100000149000000010000030300000006000004620000000100000303000000010000030200000006000004770000000100000300000000010000030200000001000002FE000000060000047A0000000100000158000000010000030500000006000004050000000100000305000000010000030400000006000004650000000100000158000000010000030700000006000003EF00000001000003070000000100000056000000010000005C000000010000030800000006000003FD0000000100000308000000010000030900000006000004050000000100000309000000010000030600000006000004650000000100000159000000010000030A00000009000000010000015D000000010000015E00000005000005EC000000010000015F0000000100000159000000010000014A000000090000000100000068000000010000030B00000009000000050000766C000000010000030B000000010000030A0000000100000149000000010000030C000000060000047E000000010000030C000000010000030D000000060000047F0000000100000302000000010000014C000000090000000100000053000000010000014D00000009000000010000030D000000010000014E0000000900000005000077F80000000100000160000000010000016100000005000004F40000000100000302000000010000014C000000090000000100000054000000010000014D000000090000000100000064000000010000014E0000000900000005000077F800000001000001620000000100000302000000010000014C000000090000000100000055000000010000014D00000009000000010000015A000000010000014E0000000900000005000077F800000001000001630000000100000302000000010000014C000000090000000100000104000000010000014D00000009000000010000015B000000010000014E0000000900000005000077F8000000010000016400000001000000170000000400004BDC0000000100000054000000010000030E000000090000000500004BF00000000100000053000000010000030E0000000900000001000003040000000100000306000000010000030E000000010000030F000000060000047E000000010000030F0000000100000310000000060000047F0000000100000302000000010000014C000000090000000100000156000000010000014D000000090000000100000310000000010000014E0000000900000005000077F8000000010000030200000001000001570000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000004D0000000100000147000000010000031100000009000000010000001E000000010000016500000001000001470000000600000414000000010000001E000000010000016600000006000004120000000100000002000000090000000800000002000000010000004D00000001000000560000000100000022000000090000000100000023000000010000031200000006000003FC00000001000003120000000400004D740000000100000023000000010000001300000006000004800000000100000002000000090000000800000002000000010000004D00000001000000500000000100000022000000090000000100000023000000010000031300000006000003FC00000001000003130000000400004DE40000000100000023000000010000001200000006000004800000000100000002000000090000000800000002000000010000004D00000001000001670000000500004D90000000010000000E000000010000031400000006000004050000000100000314000000010000005000000006000004060000000100000002000000090000000800000002000000010000004D000000010000000E000000010000031500000006000004050000000100000315000000010000005600000006000004060000000100000002000000090000000800000002000000010000004D000000010000016A0000000100000168000000010000016C0000000900000005000078FC000000010000016B000000010000031600000009000000010000016900000001000000540000000100000317000000060000048100000001000003170000000400004F30000000010000031600000001000001690000000100000317000000060000048100000001000003170000000400004F540000000100000002000000090000000800000002000000010000031600000001000001410000000100000318000000060000046700000001000003180000000400004FB0000000010000016D0000000100000168000000010000016E00000009000000050000545000000005000050680000000100000316000000010000016F000000010000031900000006000004670000000100000319000000040000500C000000010000017000000001000001680000000100000171000000090000000500006504000000050000506800000001000003160000000100000172000000010000031A0000000600000467000000010000031A000000040000506800000001000001730000000100000168000000010000017400000009000000050000674800000005000050680000000100000002000000090000000800000002000000010000004D00000001000001760000000100000175000000010000016C0000000900000005000078FC000000010000016B000000010000031B00000009000000010000031B0000000100000055000000010000031C0000000600000481000000010000031C000000040000510000000001000000020000000900000008000000020000000100000177000000010000017500000001000001710000000900000005000065040000000100000002000000090000000800000002000000010000004D0000000100000037000000010000031D0000000600000482000000010000031D0000000100000178000000010000031E0000000600000460000000010000031E00000004000051A00000000100000037000000010000005300000006000004830000000100000144000000010000031F00000006000004840000000100000037000000010000031F00000006000004850000000100000002000000090000000800000002000000010000004D0000000100000037000000010000032100000006000004820000000100000321000000010000005400000001000003200000000600000486000000010000032000000001000000530000000100000322000000060000046D000000010000032200000004000053F800000001000000370000000100000320000000010000017B0000000100000323000000010000032400000006000004870000000100000324000000040000529C00000005000052A400000005000053D00000000100000323000000010000017A0000000600000488000000010000002A000000010000032500000009000000010000017A000000010000032600000009000000010000000C000000010000017C000000010000002A0000000600000414000000010000000C000000010000017D000000010000017A0000000600000414000000010000000C000000010000017E0000000600000412000000010000000C000000010000017F0000000100000328000000060000041300000001000003280000000100000327000000090000000100000327000000040000538800000005000053A800000001000000370000000100000320000000060000048300000005000053D000000001000000560000000100000179000000090000000100000002000000090000000800000002000000010000032000000001000000540000000100000320000000060000048600000005000052240000000100000053000000010000017A00000009000000010000005000000001000001790000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000004D0000000100000180000000010000016E00000001000001820000000900000001000000540000000100000183000000090000000500007A5000000001000001810000000100000329000000090000000100000184000000010000016E00000001000001820000000900000001000000550000000100000183000000090000000500007A500000000100000181000000010000032A00000009000000010000018500000001000003290000000100000187000000090000000500007B700000000100000186000000010000032B000000090000000100000188000000010000032A0000000100000187000000090000000500007B700000000100000186000000010000032C00000009000000010000000C000000010000017C000000010000032B0000000600000414000000010000000C000000010000017D000000010000032C0000000600000414000000010000000C000000010000017E0000000600000412000000010000000C000000010000017F000000010000032E0000000600000413000000010000032E000000010000032D00000009000000010000032D000000040000560800000001000000020000000900000008000000020000000100000189000000010000016E00000001000001820000000900000001000001040000000100000183000000090000000500007A500000000100000181000000010000032F00000009000000010000018A0000000100000004000000010000033100000006000004050000000100000331000000010000018C000000090000000500007D84000000010000018B0000000100000330000000090000000100000083000000010000018D000000010000032C00000001000003320000000600000411000000010000033000000001000003320000000600000408000000010000018E00000001000003300000000100000190000000090000000100000329000000010000019100000009000000010000032A000000010000019200000009000000010000032F0000000100000193000000090000000500007DC4000000010000018F000000040000575C00000005000057E000000001000001940000000100000196000000010000032B000000010000032C000000010000034B0000000600000411000000010000034B0000000100000195000000090000000500006CA00000000100000330000000010000034C00000009000000010000034C00000006000004700000000100000002000000090000000800000002000000010000000C0000000100000197000000010000032B0000000600000414000000010000000C0000000100000198000000010000032C0000000600000414000000010000000C000000010000019900000001000003300000000600000414000000010000000C000000010000019A0000000600000412000000010000019B000000010000016E00000001000001820000000900000001000001560000000100000183000000090000000500007A5000000001000001810000000100000333000000090000000100000333000000010000033400000006000003F300000001000003340000000100000336000000060000040D00000001000003360000000100000337000000060000040E00000001000003370000000100000335000000060000040F00000001000003300000000100000335000000060000041D00000001000003300000000100000338000000060000040700000001000003380000000100000056000000010000005C000000010000033900000006000003FD0000000100000339000000010000033A00000006000004050000000100000333000000010000033B00000006000003F5000000010000033B000000010000033D000000060000040D000000010000033D000000010000033E000000060000040E000000010000033E000000010000033C000000060000040F000000010000033A000000010000033C000000060000041D000000010000019C0000000100000330000000010000033F00000006000004070000000100000333000000010000034000000006000003F600000001000003400000000100000342000000060000040D00000001000003420000000100000343000000060000040E00000001000003430000000100000341000000060000040F000000010000034100000001000000540000000100000344000000060000046700000001000003440000000400005AB800000001000000090000000100000345000000090000000500005ACC000000010000000A000000010000034500000009000000010000033F000000010000007700000009000000010000034500000001000000780000000900000005000070D4000000010000016E00000001000003470000000600000489000000010000019D000000010000016E000000010000019F00000009000000010000016E0000000100000348000000090000000500007FB80000000100000347000000010000019E000000010000034600000006000004860000000100000330000000010000034A0000000600000407000000010000034A00000001000000D8000000010000034900000006000003F000000001000003490000000100000346000000060000048A0000000100000349000000010000016E000000060000048B000000010000002E0000000400005C8C000000010000002F000000010000034D00000006000003FC000000010000034D0000000400005C24000000010000002F000000060000042A0000000500005C34000000010000002F00000006000004250000000100000349000000010000002F0000000600000426000000010000002F00000001000000A7000000010000002D00000006000004270000000100000349000000010000002F00000006000004280000000500005CA40000000100000349000000010000002D000000060000042400000001000001A0000000010000034900000001000001A1000000090000000500005CFC0000000100000330000000010000005600000006000004060000000100000002000000090000000800000002000000010000004D00000001000001A1000000010000034F00000006000003EF000000010000034F0000000100000056000000010000005C000000010000034E00000006000003FD000000010000034E00000001000001A200000006000004080000000100000350000000060000048C00000001000001A10000000100000354000000060000048D00000001000001A1000000010000002D000000060000042400000001000001A100000001000003500000000100000050000000060000048E00000001000001A100000001000003540000000600000424000000010000034E000000010000035100000006000003EF0000000100000351000000010000005C000000010000035200000006000003F000000001000003520000000100000350000000060000048F000000010000034E000000010000035300000006000004050000000100000353000000010000005600000006000004060000000100000002000000090000000800000002000000010000004D00000001000001A5000000010000016F000000010000014A00000009000000050000766C00000001000001490000000100000355000000060000047700000001000001A600000001000001A700000005000005EC00000001000000680000000100000356000000060000047B00000001000001A9000000010000016F000000010000014A00000009000000050000766C00000001000001490000000100000357000000060000047B000000010000035600000001000001A800000001000003570000000100000358000000060000047C0000000100000355000000010000014C000000090000000100000053000000010000014D000000090000000100000358000000010000014E0000000900000005000077F800000001000001AA0000000100000355000000010000014C000000090000000100000054000000010000014D0000000900000001000001A3000000010000014E0000000900000005000077F800000001000001AB0000000100000355000000010000014C000000090000000100000055000000010000014D0000000900000001000001A4000000010000014E0000000900000005000077F800000001000001AC00000001000003550000000100000147000000090000000500004CB80000000100000002000000090000000800000002000000010000004D00000001000001AF00000001000001B000000001000001AD000000010000007E00000009000000050000717C00000001000001B100000001000001AE000000010000007E00000009000000010000007D000000010000035900000009000000050000717C000000010000035900000001000001A300000009000000010000007D00000001000001A4000000090000000500005E680000000100000002000000090000000800000002000000010000004D00000001000000DA000000010000035A00000006000003FC000000010000035A00000004000061DC00000001000001B200000001000000DA00000001000001B400000009000000010000035B00000001000001B500000009000000010000035C00000001000001B600000009000000010000035D00000001000001B700000009000000050000813800000001000001B5000000010000035B0000000900000001000001B6000000010000035C0000000900000001000001B7000000010000035D0000000900000001000001B3000000010000035A00000009000000010000035A000000040000622400000001000001B8000000010000035B00000001000001A300000009000000010000035C00000001000001A4000000090000000500005E680000000100000002000000090000000800000002000000010000004D00000001000001B90000000100000172000000010000014A00000009000000050000766C0000000100000149000000010000035E000000060000047700000001000001BA00000001000001BB00000005000005EC0000000100000068000000010000035F000000060000047B00000001000001BD0000000100000172000000010000014A00000009000000050000766C00000001000001490000000100000360000000060000047B000000010000035F00000001000001BC00000001000003600000000100000361000000060000047C000000010000035E000000010000014C000000090000000100000053000000010000014D000000090000000100000361000000010000014E0000000900000005000077F800000001000001BE00000001000001BF00000005000004F4000000010000035E000000010000014C000000090000000100000054000000010000014D000000090000000100000064000000010000014E0000000900000005000077F800000001000001C000000001000001C100000005000006C4000000010000035E000000010000014C000000090000000100000104000000010000014D00000009000000010000006B000000010000014E0000000900000005000077F800000001000001C2000000010000035E0000000100000147000000090000000500004CB80000000100000002000000090000000800000002000000010000004D00000001000001C30000000100000362000000010000017A0000000900000005000051EC000000010000017A0000000100000362000000090000000100000179000000040000649C00000005000064B0000000010000000200000009000000080000000200000001000001C4000000010000002A00000001000001AD00000009000000010000036200000001000001AE0000000900000005000060540000000100000002000000090000000800000002000000010000004D00000001000001710000000100000363000000060000048900000001000001C5000000010000016F000000010000014A000000090000000100000171000000010000036400000009000000050000766C000000010000036300000001000001490000000100000365000000060000045B00000001000003650000000400006598000000010000000200000009000000080000000200000001000001C6000000010000017100000001000001820000000900000001000000540000000100000183000000090000000500007A50000000010000018100000001000003660000000900000001000001C7000000010000017100000001000001820000000900000001000000550000000100000183000000090000000500007A50000000010000018100000001000003670000000900000001000001C800000001000003660000000100000187000000090000000500007B70000000010000018600000001000003680000000900000001000001C900000001000003670000000100000187000000090000000500007B700000000100000186000000010000036900000009000000010000000C00000001000001CA00000001000003680000000600000414000000010000000C00000001000001CB00000001000003690000000600000414000000010000000C00000001000001CC0000000600000412000000010000000C00000001000001CD000000010000036B0000000600000413000000010000036B000000010000036A000000090000000100000002000000090000000800000002000000010000004D0000000100000174000000010000036C000000060000048900000001000001CE0000000100000172000000010000014A000000090000000100000174000000010000036D00000009000000050000766C000000010000036C0000000100000149000000010000036E000000060000045B000000010000036E00000004000067DC000000010000000200000009000000080000000200000001000001CF000000010000017400000001000001820000000900000001000001040000000100000183000000090000000500007A500000000100000181000000010000036F0000000900000001000001D000000001000001D100000005000004F400000001000000640000000100000187000000090000000500007B700000000100000186000000010000037000000009000000010000000C00000001000001D200000001000003700000000600000414000000010000000C00000001000001D3000000010000036F0000000600000414000000010000000C00000001000001D40000000600000412000000010000000C00000001000001D50000000100000372000000060000041300000001000003720000000100000371000000090000000100000002000000090000000800000002000000010000004D000000010000000E0000000100000374000000060000043F00000001000001D6000000010000000E00000001000003750000000900000005000000380000000100000374000000010000004E000000010000004900000001000001D700000001000001D80000000100000373000000060000045A0000000100000053000000010000037700000009000000010000037700000001000003730000000100000378000000060000045B00000001000003780000000400006BCC000000010000004900000001000003770000000100000379000000060000045C0000000100000379000000010000037A00000006000003FC000000010000037A0000000400006B70000000010000037900000001000001D9000000010000037C0000000600000490000000010000037C000000010000037600000006000004890000000100000053000000010000037D00000009000000010000037D0000000100000376000000010000037E000000060000045B000000010000037E0000000400006B70000000010000037C000000010000037D000000010000037F000000060000045C000000010000037F000000010000038000000006000003FC00000001000003800000000400006AE40000000500006AEC0000000500006B48000000010000037F00000001000003810000000900000001000003810000000100000382000000060000049100000001000003820000000400006B300000000500006B48000000010000038100000001000001DA0000000600000412000000010000037D0000000100000054000000010000037D00000006000004620000000500006A640000000100000067000000010000037B0000000900000001000000490000000100000377000000010000037B0000000600000461000000010000037700000001000000540000000100000377000000060000046200000005000069A000000001000000490000000100000376000000060000048900000001000000490000000100000053000000010000037600000006000004920000000100000002000000090000000800000002000000010000004D00000001000001DC0000000500006E0000000001000001DB00000001000001DD00000001000000BB0000000100000383000000060000041100000001000001DE0000000100000384000000090000000100000383000000010000038400000006000004930000000100000002000000090000000800000002000000010000004D00000001000001DF0000000500006E0000000001000001DB00000001000001DD00000001000001950000000100000385000000060000041100000001000001E00000000100000386000000090000000100000385000000010000038600000006000004940000000100000002000000090000000800000002000000010000004D00000001000001E10000000500006E0000000001000001DB00000001000001DD00000001000000F60000000100000387000000060000041100000001000001E20000000100000388000000090000000100000387000000010000038800000006000004950000000100000002000000090000000800000002000000010000004D00000001000001E600000001000001E400000001000001E80000000900000005000083E000000001000001E500000001000001E700000001000001E300000006000004030000000100000002000000090000000800000002000000010000004D000000010000004B0000000100000389000000060000040B0000000100000389000000010000038A000000060000042F000000010000038A0000000400006E5C000000010000004B00000001000001DD000000090000000500006F5400000001000001EA000000010000004A00000001000001E4000000090000000500006DA000000001000001E9000000010000005300000001000001E3000000060000046100000001000001E900000001000000540000000100000095000000060000046100000001000001E9000000010000005500000001000001EB000000060000046100000001000001E9000000010000010400000001000001EC000000060000046100000001000001E9000000010000015600000001000001ED000000060000046100000001000001EE00000001000001E9000000010000004B0000000600000496000000010000004B00000001000001DD000000090000000100000002000000090000000800000002000000010000006E000000010000038B00000009000000010000038B00000001000001EF000000010000038C0000000600000497000000010000038C00000001000001EF000000010000038D0000000600000497000000010000038B00000001000001EF000000010000038E0000000600000498000000010000038E000000010000038F000000060000047B000000010000038C00000001000001EF0000000100000390000000060000049800000001000003900000000100000391000000060000047B000000010000038D00000001000001EF0000000100000392000000060000049800000001000003920000000100000393000000060000047B000000010000038F000000010000039100000001000003930000000100000394000000060000047C000000010000039400000001000001BC000000010000006D0000000600000499000000010000000200000009000000080000000200000001000000020000000900000008000000020000000100000077000000010000039500000006000003FC000000010000039500000004000071040000000500007118000000010000000200000009000000080000000200000001000000770000000100000078000000060000049A000000010000007700000001000000C700000001000000C8000000060000043E0000000100000077000000010000009A00000006000004190000000100000002000000090000000800000002000000010000007E00000001000001F00000000100000396000000060000049B000000010000039600000001000001F10000000100000397000000060000046600000001000003970000000100000398000000060000047B000000010000007E00000001000001F20000000100000399000000060000049B000000010000039900000001000001F3000000010000039A0000000600000466000000010000039A000000010000039B000000060000047B000000010000007E00000001000001F3000000010000039C0000000600000466000000010000039C000000010000039D000000060000047B0000000100000398000000010000039B000000010000039D000000010000007D000000060000047C0000000100000002000000090000000800000002000000010000008900000001000001D9000000010000039F0000000600000490000000010000039F000000010000039E0000000900000001000001F400000001000003A000000009000000010000039E00000001000003A10000000600000489000000010000005300000001000003A20000000900000001000003A200000001000003A100000001000003A3000000060000045B00000001000003A300000004000074CC000000010000039E00000001000003A200000001000003A4000000060000045C00000001000003A400000001000001F500000001000003A5000000060000049C000000010000006700000001000003A60000000900000001000003A500000001000003A600000001000003A7000000060000046900000001000003A700000004000073D000000005000074A400000001000003A400000001000001F500000001000003A8000000060000041300000001000003A8000000010000006700000001000003A9000000060000049D00000001000003A9000000040000745800000001000003A800000001000003AA000000060000049E00000001000003AA00000001000003A000000001000003A9000000060000049F00000001000003A900000004000074A400000001000003A400000001000003AB0000000900000001000003AB000000010000008800000009000000010000000200000009000000080000000200000001000003A2000000010000005400000001000003A20000000600000462000000050000731400000001000000670000000100000088000000090000000100000002000000090000000800000002000000010000000200000009000000080000000200000001000000FA000000010000005300000001000003AC000000060000046D00000001000003AC000000040000763000000001000000FA000000010000010400000001000003AD000000060000046E00000001000003AD000000040000763000000001000001F600000001000000FA00000001000003AE000000060000046F00000008000003AE000000010000010D00000001000000F900000009000000010000000200000009000000080000000200000001000001F700000001000000F9000000090000000100000002000000090000000800000002000000010000010A00000001000000F900000009000000010000000200000009000000080000000200000001000001F800000001000000F900000009000000010000000200000009000000080000000200000001000001F900000001000000F90000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000014A000000010000005300000001000003AF000000060000046D00000001000003AF00000004000077BC000000010000014A000000010000015600000001000003B0000000060000046E00000001000003B000000004000077BC00000001000001FA000000010000014A00000001000003B1000000060000046F00000008000003B100000001000001FB000000010000014900000009000000010000000200000009000000080000000200000001000001560000000100000149000000090000000100000002000000090000000800000002000000010000015600000001000001490000000900000001000000020000000900000008000000020000000100000156000000010000014900000009000000010000000200000009000000080000000200000001000000530000000100000149000000090000000100000002000000090000000800000002000000010000005300000001000001490000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000014C000000010000006700000001000003B2000000060000049D00000001000003B20000000400007860000000010000014C00000001000003B3000000060000048900000001000003B3000000010000014D00000001000003B2000000060000046000000001000003B200000004000078E8000000010000014C00000001000003B4000000060000048900000001000003B4000000010000005400000001000003B5000000060000048600000001000003B5000000010000014D00000001000003B60000000600000486000000010000014C00000001000003B6000000010000014E00000006000004A00000000100000002000000090000000800000002000000010000016C000000010000006700000001000003B7000000060000049D00000001000003B70000000400007964000000010000016C00000001000003B8000000060000048900000001000003B8000000010000005300000001000003B7000000060000046000000001000003B70000000400007A2800000001000001FC000000010000016C00000001000001820000000900000001000000530000000100000183000000090000000500007A50000000010000018100000001000003B900000006000003F500000001000003B900000001000003BB000000060000040D00000001000003BB00000001000003BC000000060000040E00000001000003BC00000001000003BA000000060000040F00000001000003BA000000010000016B000000090000000500007A3C00000001000001FD000000010000016B0000000900000001000000020000000900000008000000020000000100000182000000010000006700000001000003BD000000060000049D00000001000003BD0000000400007AB8000000010000018200000001000003BE000000060000048900000001000003BE000000010000018300000001000003BD000000060000046000000001000003BD0000000400007B48000000010000018200000001000003BF000000060000048900000001000003BF000000010000005400000001000003C0000000060000048600000001000003C0000000010000018300000001000003C10000000600000486000000010000018200000001000003C1000000010000018100000006000004A10000000500007B5C00000001000000C70000000100000181000000090000000100000002000000090000000800000002000000010000018700000001000003C200000006000003F300000001000003C200000001000003C4000000060000040D00000001000003C400000001000003C5000000060000040E00000001000003C500000001000003C3000000060000040F00000001000003C300000001000001F100000001000003C6000000060000046600000001000003C600000001000001F000000001000003C7000000060000041C000000010000018700000001000003C800000006000003F500000001000003C800000001000003CA000000060000040D00000001000003CA00000001000003CB000000060000040E00000001000003CB00000001000003C9000000060000040F00000001000003C900000001000001F300000001000003CC000000060000046600000001000003CC00000001000001F200000001000003CD000000060000041C00000001000003C700000001000003CD00000001000003CE00000006000004A2000000010000018700000001000003CF00000006000003F600000001000003CF00000001000003D1000000060000040D00000001000003D100000001000003D2000000060000040E00000001000003D200000001000003D0000000060000040F00000001000003D000000001000001F300000001000003D3000000060000046600000001000003CE00000001000003D3000000010000018600000006000004A20000000100000002000000090000000800000002000000010000018C000000010000018B00000006000004A300000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000019000000001000003D400000006000003FC00000001000003D40000000400007DF40000000500007E1C0000000100000050000000010000018F000000090000000100000002000000090000000800000002000000010000019000000001000003D5000000060000040700000001000003D500000001000003D600000006000004A400000001000003D6000000010000005500000001000003D7000000060000045B00000001000003D70000000400007EA40000000100000050000000010000018F000000090000000100000002000000090000000800000002000000010000019000000001000003D9000000060000040700000001000003D9000000010000005400000001000003D800000006000004A500000001000003D800000001000003DA00000006000003FC00000001000003DA0000000400007F0C0000000500007F340000000100000050000000010000018F00000009000000010000000200000009000000080000000200000001000003D8000000010000019100000006000004A600000001000003D80000000100000192000000060000041900000001000003D8000000010000019300000006000004A70000000100000056000000010000018F0000000900000001000000020000000900000008000000020000000100000002000000090000000800000002000000010000019F000000010000006700000001000003DB000000060000049D00000001000003DB0000000400008020000000010000019F00000001000003DC000000060000048900000001000003DC000000010000005300000001000003DB000000060000046000000001000003DB000000040000811000000001000001FE000000010000019F00000001000001820000000900000001000000530000000100000183000000090000000500007A50000000010000018100000001000003DD00000006000003F600000001000003DD00000001000003DF000000060000040D00000001000003DF00000001000003E0000000060000040E00000001000003E000000001000003DE000000060000040F000000010000019F00000001000003E1000000060000048900000001000003DE000000010000005300000001000003E1000000010000019E00000006000004A800000005000081240000000100000053000000010000019E00000009000000010000000200000009000000080000000200000001000001B400000001000003E200000006000003FC00000001000003E2000000040000816800000005000081CC00000001000000C700000001000001B50000000900000001000000C700000001000001B60000000900000001000000C700000001000001B700000009000000010000005000000001000001B300000009000000010000000200000009000000080000000200000001000001B400000001000003E3000000060000040700000001000003E300000001000003E400000006000004A400000001000003E4000000010000005500000001000003E5000000060000045B00000001000003E5000000040000829000000001000000C700000001000001B50000000900000001000000C700000001000001B60000000900000001000000C700000001000001B700000009000000010000005000000001000001B300000009000000010000000200000009000000080000000200000001000001B400000001000003E7000000060000040700000001000003E7000000010000005400000001000003E600000006000004A500000001000003E600000001000003E800000006000003FC00000001000003E800000004000082F8000000050000835C00000001000000C700000001000001B50000000900000001000000C700000001000001B60000000900000001000000C700000001000001B700000009000000010000005000000001000001B300000009000000010000000200000009000000080000000200000001000003E600000001000001B500000006000004A900000001000003E600000001000001B6000000060000041700000001000003E600000001000001B700000006000004AA000000010000005600000001000001B3000000090000000100000002000000090000000800000002000000010000000200000009000000080000000200000001000001E800000001000001FF00000001000001E800000006000004AB00000001000001E800000001000003E900000006000004AC00000001000003E900000001000003EA00000006000004AD00000001000001E800000001000003EB00000006000004AE00000001000003EB00000001000003EC00000006000004AD00000001000001E800000001000003ED00000006000004AF00000001000003ED00000001000003EE00000006000004AD000000010000020000000001000003EA00000001000003EC00000001000003EE00000001000001E700000006000004100000000100000002000000090000000800000002000000010000000200000009000000080000000200000001000000670000000100000201000000090000000100000002000000090000000800000002",
  "byteCodeLength": 34064,
  "symbols": {
    "__35__intnlparam": {
      "name": "__35__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 439
    },
    "__const_SystemSingle_8": {
      "name": "__const_SystemSingle_8",
      "type": "System.Single",
      "address": 211
    },
    "__intnl_UnityEngineVector2_2": {
      "name": "__intnl_UnityEngineVector2_2",
      "type": "UnityEngine.Vector2",
      "address": 626
    },
    "__intnl_SystemBoolean_83": {
      "name": "__intnl_SystemBoolean_83",
      "type": "System.Boolean",
      "address": 858
    },
    "__intnl_SystemBoolean_93": {
      "name": "__intnl_SystemBoolean_93",
      "type": "System.Boolean",
      "address": 905
    },
    "__intnl_SystemBoolean_13": {
      "name": "__intnl_SystemBoolean_13",
      "type": "System.Boolean",
      "address": 594
    },
    "__intnl_SystemBoolean_23": {
      "name": "__intnl_SystemBoolean_23",
      "type": "System.Boolean",
      "address": 634
    },
    "__intnl_SystemBoolean_33": {
      "name": "__intnl_SystemBoolean_33",
      "type": "System.Boolean",
      "address": 671
    },
    "__intnl_SystemBoolean_43": {
      "name": "__intnl_SystemBoolean_43",
      "type": "System.Boolean",
      "address": 705
    },
    "__intnl_SystemBoolean_53": {
      "name": "__intnl_SystemBoolean_53",
      "type": "System.Boolean",
      "address": 724
    },
    "__intnl_SystemBoolean_63": {
      "name": "__intnl_SystemBoolean_63",
      "type": "System.Boolean",
      "address": 735
    },
    "__intnl_SystemBoolean_73": {
      "name": "__intnl_SystemBoolean_73",
      "type": "System.Boolean",
      "address": 793
    },
    "__13__intnlparam": {
      "name": "__13__intnlparam",
      "type": "System.Int32",
      "address": 333
    },
    "__gintnl_SystemUInt32_96": {
      "name": "__gintnl_SystemUInt32_96",
      "type": "System.UInt32",
      "address": 351
    },
    "__gintnl_SystemUInt32_86": {
      "name": "__gintnl_SystemUInt32_86",
      "type": "System.UInt32",
      "address": 335
    },
    "__gintnl_SystemUInt32_56": {
      "name": "__gintnl_SystemUInt32_56",
      "type": "System.UInt32",
      "address": 286
    },
    "__gintnl_SystemUInt32_46": {
      "name": "__gintnl_SystemUInt32_46",
      "type": "System.UInt32",
      "address": 267
    },
    "__gintnl_SystemUInt32_76": {
      "name": "__gintnl_SystemUInt32_76",
      "type": "System.UInt32",
      "address": 309
    },
    "__gintnl_SystemUInt32_66": {
      "name": "__gintnl_SystemUInt32_66",
      "type": "System.UInt32",
      "address": 296
    },
    "__gintnl_SystemUInt32_16": {
      "name": "__gintnl_SystemUInt32_16",
      "type": "System.UInt32",
      "address": 148
    },
    "__gintnl_SystemUInt32_36": {
      "name": "__gintnl_SystemUInt32_36",
      "type": "System.UInt32",
      "address": 221
    },
    "__gintnl_SystemUInt32_26": {
      "name": "__gintnl_SystemUInt32_26",
      "type": "System.UInt32",
      "address": 186
    },
    "__lcl_closestPoint_UnityEngineVector3_0": {
      "name": "__lcl_closestPoint_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 632
    },
    "__intnl_SystemSingle_20": {
      "name": "__intnl_SystemSingle_20",
      "type": "System.Single",
      "address": 758
    },
    "__intnl_SystemSingle_21": {
      "name": "__intnl_SystemSingle_21",
      "type": "System.Single",
      "address": 759
    },
    "__intnl_SystemSingle_22": {
      "name": "__intnl_SystemSingle_22",
      "type": "System.Single",
      "address": 761
    },
    "__intnl_SystemSingle_23": {
      "name": "__intnl_SystemSingle_23",
      "type": "System.Single",
      "address": 762
    },
    "__intnl_SystemSingle_24": {
      "name": "__intnl_SystemSingle_24",
      "type": "System.Single",
      "address": 763
    },
    "__intnl_SystemSingle_25": {
      "name": "__intnl_SystemSingle_25",
      "type": "System.Single",
      "address": 820
    },
    "__intnl_SystemSingle_26": {
      "name": "__intnl_SystemSingle_26",
      "type": "System.Single",
      "address": 827
    },
    "__intnl_SystemSingle_27": {
      "name": "__intnl_SystemSingle_27",
      "type": "System.Single",
      "address": 832
    },
    "__intnl_SystemSingle_28": {
      "name": "__intnl_SystemSingle_28",
      "type": "System.Single",
      "address": 854
    },
    "__intnl_SystemSingle_29": {
      "name": "__intnl_SystemSingle_29",
      "type": "System.Single",
      "address": 855
    },
    "__intnl_SystemInt32_15": {
      "name": "__intnl_SystemInt32_15",
      "type": "System.Int32",
      "address": 782
    },
    "__intnl_SystemInt32_35": {
      "name": "__intnl_SystemInt32_35",
      "type": "System.Int32",
      "address": 924
    },
    "__intnl_SystemInt32_25": {
      "name": "__intnl_SystemInt32_25",
      "type": "System.Int32",
      "address": 876
    },
    "__intnl_SystemInt32_55": {
      "name": "__intnl_SystemInt32_55",
      "type": "System.Int32",
      "address": 976
    },
    "__intnl_SystemInt32_45": {
      "name": "__intnl_SystemInt32_45",
      "type": "System.Int32",
      "address": 959
    },
    "__const_SystemInt64_0": {
      "name": "__const_SystemInt64_0",
      "type": "System.Int64",
      "address": 500
    },
    "__lcl_ownerIdVector_UnityEngineVector3_0": {
      "name": "__lcl_ownerIdVector_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 879
    },
    "__lcl_positionCount_SystemInt32_2": {
      "name": "__lcl_positionCount_SystemInt32_2",
      "type": "System.Int32",
      "address": 838
    },
    "__lcl_positionCount_SystemInt32_0": {
      "name": "__lcl_positionCount_SystemInt32_0",
      "type": "System.Int32",
      "address": 750
    },
    "__lcl_positionCount_SystemInt32_1": {
      "name": "__lcl_positionCount_SystemInt32_1",
      "type": "System.Int32",
      "address": 766
    },
    "_pointerRadius": {
      "name": "_pointerRadius",
      "type": "System.Single",
      "address": 16
    },
    "allowCallPen": {
      "name": "allowCallPen",
      "type": "System.Boolean",
      "address": 11
    },
    "__lcl_inkIdToken_VRCSDK3DataDataToken_0": {
      "name": "__lcl_inkIdToken_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 803
    },
    "__intnl_SystemString_1": {
      "name": "__intnl_SystemString_1",
      "type": "System.String",
      "address": 549
    },
    "__lcl_inkId_SystemInt32_2": {
      "name": "__lcl_inkId_SystemInt32_2",
      "type": "System.Int32",
      "address": 866
    },
    "__lcl_inkId_SystemInt32_3": {
      "name": "__lcl_inkId_SystemInt32_3",
      "type": "System.Int32",
      "address": 873
    },
    "__lcl_inkId_SystemInt32_0": {
      "name": "__lcl_inkId_SystemInt32_0",
      "type": "System.Int32",
      "address": 741
    },
    "__lcl_inkId_SystemInt32_1": {
      "name": "__lcl_inkId_SystemInt32_1",
      "type": "System.Int32",
      "address": 812
    },
    "currentState": {
      "name": "currentState",
      "type": "System.Int32",
      "address": 40
    },
    "__6__intnlparam": {
      "name": "__6__intnlparam",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 136
    },
    "__lcl_inkInfo_UnityEngineVector3_0": {
      "name": "__lcl_inkInfo_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 819
    },
    "__intnl_SystemDouble_18": {
      "name": "__intnl_SystemDouble_18",
      "type": "System.Double",
      "address": 977
    },
    "__intnl_SystemDouble_19": {
      "name": "__intnl_SystemDouble_19",
      "type": "System.Double",
      "address": 978
    },
    "__intnl_SystemDouble_14": {
      "name": "__intnl_SystemDouble_14",
      "type": "System.Double",
      "address": 964
    },
    "__intnl_SystemDouble_15": {
      "name": "__intnl_SystemDouble_15",
      "type": "System.Double",
      "address": 965
    },
    "__intnl_SystemDouble_16": {
      "name": "__intnl_SystemDouble_16",
      "type": "System.Double",
      "address": 970
    },
    "__intnl_SystemDouble_17": {
      "name": "__intnl_SystemDouble_17",
      "type": "System.Double",
      "address": 971
    },
    "__intnl_SystemDouble_10": {
      "name": "__intnl_SystemDouble_10",
      "type": "System.Double",
      "address": 834
    },
    "__intnl_SystemDouble_11": {
      "name": "__intnl_SystemDouble_11",
      "type": "System.Double",
      "address": 835
    },
    "__intnl_SystemDouble_12": {
      "name": "__intnl_SystemDouble_12",
      "type": "System.Double",
      "address": 955
    },
    "__intnl_SystemDouble_13": {
      "name": "__intnl_SystemDouble_13",
      "type": "System.Double",
      "address": 956
    },
    "__lcl_udonComponent_UnityEngineComponent_0": {
      "name": "__lcl_udonComponent_UnityEngineComponent_0",
      "type": "UnityEngine.Component",
      "address": 895
    },
    "__8__intnlparam": {
      "name": "__8__intnlparam",
      "type": "System.String",
      "address": 249
    },
    "__const_SystemType_1": {
      "name": "__const_SystemType_1",
      "type": "System.Type",
      "address": 92
    },
    "__const_SystemString_46": {
      "name": "__const_SystemString_46",
      "type": "System.String",
      "address": 255
    },
    "__const_SystemString_47": {
      "name": "__const_SystemString_47",
      "type": "System.String",
      "address": 258
    },
    "__const_SystemString_44": {
      "name": "__const_SystemString_44",
      "type": "System.String",
      "address": 251
    },
    "__const_SystemString_45": {
      "name": "__const_SystemString_45",
      "type": "System.String",
      "address": 253
    },
    "__const_SystemString_42": {
      "name": "__const_SystemString_42",
      "type": "System.String",
      "address": 242
    },
    "__const_SystemString_43": {
      "name": "__const_SystemString_43",
      "type": "System.String",
      "address": 247
    },
    "__const_SystemString_40": {
      "name": "__const_SystemString_40",
      "type": "System.String",
      "address": 233
    },
    "__const_SystemString_41": {
      "name": "__const_SystemString_41",
      "type": "System.String",
      "address": 235
    },
    "__const_SystemString_48": {
      "name": "__const_SystemString_48",
      "type": "System.String",
      "address": 265
    },
    "__const_SystemString_49": {
      "name": "__const_SystemString_49",
      "type": "System.String",
      "address": 266
    },
    "__intnl_SystemSingle_0": {
      "name": "__intnl_SystemSingle_0",
      "type": "System.Single",
      "address": 518
    },
    "__gintnl_SystemUInt32_160": {
      "name": "__gintnl_SystemUInt32_160",
      "type": "System.UInt32",
      "address": 510
    },
    "pointerRenderer": {
      "name": "pointerRenderer",
      "type": "UnityEngine.Renderer",
      "address": 35
    },
    "__intnl_SystemObject_0": {
      "name": "__intnl_SystemObject_0",
      "type": "System.Object",
      "address": 552
    },
    "syncer": {
      "name": "syncer",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 13
    },
    "inkPrefab": {
      "name": "inkPrefab",
      "type": "UnityEngine.LineRenderer",
      "address": 4
    },
    "__32__intnlparam": {
      "name": "__32__intnlparam",
      "type": "UnityEngine.GameObject",
      "address": 436
    },
    "__lcl_s_UnityEngineVector3_0": {
      "name": "__lcl_s_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 517
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 116
    },
    "__27__intnlparam": {
      "name": "__27__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 402
    },
    "enabledLateSync": {
      "name": "enabledLateSync",
      "type": "System.Boolean",
      "address": 23
    },
    "__const_UnityEngineKeyCode_4": {
      "name": "__const_UnityEngineKeyCode_4",
      "type": "UnityEngine.KeyCode",
      "address": 184
    },
    "__intnl_UnityEngineGameObject_19": {
      "name": "__intnl_UnityEngineGameObject_19",
      "type": "UnityEngine.GameObject",
      "address": 851
    },
    "__intnl_UnityEngineTransform_0": {
      "name": "__intnl_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 516
    },
    "__lcl_shader_UnityEngineShader_0": {
      "name": "__lcl_shader_UnityEngineShader_0",
      "type": "UnityEngine.Shader",
      "address": 583
    },
    "onTriggerEnterOther": {
      "name": "onTriggerEnterOther",
      "type": "UnityEngine.Collider",
      "address": 219
    },
    "__const_SystemSingle_0": {
      "name": "__const_SystemSingle_0",
      "type": "System.Single",
      "address": 81
    },
    "__0_ownerIdVector__param": {
      "name": "__0_ownerIdVector__param",
      "type": "UnityEngine.Vector3",
      "address": 320
    },
    "__intnl_SystemDouble_3": {
      "name": "__intnl_SystemDouble_3",
      "type": "System.Double",
      "address": 544
    },
    "__0___0_ColorBeginTag__ret": {
      "name": "__0___0_ColorBeginTag__ret",
      "type": "System.String",
      "address": 483
    },
    "__intnl_UnityEngineTrailRenderer_0": {
      "name": "__intnl_UnityEngineTrailRenderer_0",
      "type": "UnityEngine.TrailRenderer",
      "address": 747
    },
    "__intnl_UnityEngineVector3_6": {
      "name": "__intnl_UnityEngineVector3_6",
      "type": "UnityEngine.Vector3",
      "address": 616
    },
    "prevClickPos": {
      "name": "prevClickPos",
      "type": "UnityEngine.Vector3",
      "address": 39
    },
    "_isCheckedLocalPlayerId": {
      "name": "_isCheckedLocalPlayerId",
      "type": "System.Boolean",
      "address": 49
    },
    "__intnl_SystemBoolean_86": {
      "name": "__intnl_SystemBoolean_86",
      "type": "System.Boolean",
      "address": 878
    },
    "__intnl_SystemBoolean_96": {
      "name": "__intnl_SystemBoolean_96",
      "type": "System.Boolean",
      "address": 931
    },
    "__intnl_SystemBoolean_16": {
      "name": "__intnl_SystemBoolean_16",
      "type": "System.Boolean",
      "address": 597
    },
    "__intnl_SystemBoolean_26": {
      "name": "__intnl_SystemBoolean_26",
      "type": "System.Boolean",
      "address": 652
    },
    "__intnl_SystemBoolean_36": {
      "name": "__intnl_SystemBoolean_36",
      "type": "System.Boolean",
      "address": 680
    },
    "__intnl_SystemBoolean_46": {
      "name": "__intnl_SystemBoolean_46",
      "type": "System.Boolean",
      "address": 713
    },
    "__intnl_SystemBoolean_56": {
      "name": "__intnl_SystemBoolean_56",
      "type": "System.Boolean",
      "address": 727
    },
    "__intnl_SystemBoolean_66": {
      "name": "__intnl_SystemBoolean_66",
      "type": "System.Boolean",
      "address": 751
    },
    "__intnl_SystemBoolean_76": {
      "name": "__intnl_SystemBoolean_76",
      "type": "System.Boolean",
      "address": 798
    },
    "__1_inkId__param": {
      "name": "__1_inkId__param",
      "type": "System.Int32",
      "address": 378
    },
    "__lcl_deltaDistance_SystemSingle_0": {
      "name": "__lcl_deltaDistance_SystemSingle_0",
      "type": "System.Single",
      "address": 635
    },
    "__const_SystemInt32_11": {
      "name": "__const_SystemInt32_11",
      "type": "System.Int32",
      "address": 370
    },
    "__intnl_SystemObject_15": {
      "name": "__intnl_SystemObject_15",
      "type": "System.Object",
      "address": 882
    },
    "__intnl_SystemObject_14": {
      "name": "__intnl_SystemObject_14",
      "type": "System.Object",
      "address": 875
    },
    "__intnl_SystemObject_13": {
      "name": "__intnl_SystemObject_13",
      "type": "System.Object",
      "address": 814
    },
    "__intnl_SystemObject_12": {
      "name": "__intnl_SystemObject_12",
      "type": "System.Object",
      "address": 808
    },
    "__intnl_SystemObject_11": {
      "name": "__intnl_SystemObject_11",
      "type": "System.Object",
      "address": 744
    },
    "__intnl_SystemObject_10": {
      "name": "__intnl_SystemObject_10",
      "type": "System.Object",
      "address": 692
    },
    "inkPosition": {
      "name": "inkPosition",
      "type": "UnityEngine.Transform",
      "address": 5
    },
    "__intnl_SystemSingle_10": {
      "name": "__intnl_SystemSingle_10",
      "type": "System.Single",
      "address": 609
    },
    "__intnl_SystemSingle_11": {
      "name": "__intnl_SystemSingle_11",
      "type": "System.Single",
      "address": 619
    },
    "__intnl_SystemSingle_12": {
      "name": "__intnl_SystemSingle_12",
      "type": "System.Single",
      "address": 621
    },
    "__intnl_SystemSingle_13": {
      "name": "__intnl_SystemSingle_13",
      "type": "System.Single",
      "address": 622
    },
    "__intnl_SystemSingle_14": {
      "name": "__intnl_SystemSingle_14",
      "type": "System.Single",
      "address": 623
    },
    "__intnl_SystemSingle_15": {
      "name": "__intnl_SystemSingle_15",
      "type": "System.Single",
      "address": 697
    },
    "__intnl_SystemSingle_16": {
      "name": "__intnl_SystemSingle_16",
      "type": "System.Single",
      "address": 698
    },
    "__intnl_SystemSingle_17": {
      "name": "__intnl_SystemSingle_17",
      "type": "System.Single",
      "address": 700
    },
    "__intnl_SystemSingle_18": {
      "name": "__intnl_SystemSingle_18",
      "type": "System.Single",
      "address": 703
    },
    "__intnl_SystemSingle_19": {
      "name": "__intnl_SystemSingle_19",
      "type": "System.Single",
      "address": 757
    },
    "_localPlayerId": {
      "name": "_localPlayerId",
      "type": "System.Int32",
      "address": 50
    },
    "__intnl_SystemInt32_12": {
      "name": "__intnl_SystemInt32_12",
      "type": "System.Int32",
      "address": 769
    },
    "__intnl_SystemInt32_32": {
      "name": "__intnl_SystemInt32_32",
      "type": "System.Int32",
      "address": 919
    },
    "__intnl_SystemInt32_22": {
      "name": "__intnl_SystemInt32_22",
      "type": "System.Int32",
      "address": 833
    },
    "__intnl_SystemInt32_52": {
      "name": "__intnl_SystemInt32_52",
      "type": "System.Int32",
      "address": 972
    },
    "__intnl_SystemInt32_42": {
      "name": "__intnl_SystemInt32_42",
      "type": "System.Int32",
      "address": 952
    },
    "__intnl_SystemInt32_62": {
      "name": "__intnl_SystemInt32_62",
      "type": "System.Int32",
      "address": 1002
    },
    "__0___0_PackData__ret": {
      "name": "__0___0_PackData__ret",
      "type": "UnityEngine.Vector3[]",
      "address": 316
    },
    "__10__intnlparam": {
      "name": "__10__intnlparam",
      "type": "System.Int32",
      "address": 329
    },
    "__intnl_SystemString_9": {
      "name": "__intnl_SystemString_9",
      "type": "System.String",
      "address": 818
    },
    "__lcl_y_SystemInt32_0": {
      "name": "__lcl_y_SystemInt32_0",
      "type": "System.Int32",
      "address": 908
    },
    "__gintnl_SystemUInt32Array_0": {
      "name": "__gintnl_SystemUInt32Array_0",
      "type": "System.UInt32[]",
      "address": 261
    },
    "__intnl_UnityEngineVector3Int_0": {
      "name": "__intnl_UnityEngineVector3Int_0",
      "type": "UnityEngine.Vector3Int",
      "address": 780
    },
    "__intnl_UnityEngineVector3Int_1": {
      "name": "__intnl_UnityEngineVector3Int_1",
      "type": "UnityEngine.Vector3Int",
      "address": 783
    },
    "__this_VRCUdonUdonBehaviour_12": {
      "name": "__this_VRCUdonUdonBehaviour_12",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 254
    },
    "pointerMaterialActive": {
      "name": "pointerMaterialActive",
      "type": "UnityEngine.Material",
      "address": 19
    },
    "__intnl_SystemSingle_8": {
      "name": "__intnl_SystemSingle_8",
      "type": "System.Single",
      "address": 545
    },
    "__gintnl_SystemUInt32_9": {
      "name": "__gintnl_SystemUInt32_9",
      "type": "System.UInt32",
      "address": 121
    },
    "__gintnl_SystemUInt32_8": {
      "name": "__gintnl_SystemUInt32_8",
      "type": "System.UInt32",
      "address": 118
    },
    "__gintnl_SystemUInt32_5": {
      "name": "__gintnl_SystemUInt32_5",
      "type": "System.UInt32",
      "address": 112
    },
    "__gintnl_SystemUInt32_4": {
      "name": "__gintnl_SystemUInt32_4",
      "type": "System.UInt32",
      "address": 111
    },
    "__gintnl_SystemUInt32_7": {
      "name": "__gintnl_SystemUInt32_7",
      "type": "System.UInt32",
      "address": 115
    },
    "__gintnl_SystemUInt32_6": {
      "name": "__gintnl_SystemUInt32_6",
      "type": "System.UInt32",
      "address": 113
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 105
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 88
    },
    "__gintnl_SystemUInt32_3": {
      "name": "__gintnl_SystemUInt32_3",
      "type": "System.UInt32",
      "address": 108
    },
    "__gintnl_SystemUInt32_2": {
      "name": "__gintnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 106
    },
    "__5_data__param": {
      "name": "__5_data__param",
      "type": "UnityEngine.Vector3[]",
      "address": 360
    },
    "__intnl_SystemDouble_20": {
      "name": "__intnl_SystemDouble_20",
      "type": "System.Double",
      "address": 991
    },
    "__intnl_SystemDouble_21": {
      "name": "__intnl_SystemDouble_21",
      "type": "System.Double",
      "address": 992
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 80
    },
    "__const_SystemBoolean_1": {
      "name": "__const_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 86
    },
    "__0_value__param": {
      "name": "__0_value__param",
      "type": "UnityEngine.Vector3",
      "address": 101
    },
    "__const_SystemType_2": {
      "name": "__const_SystemType_2",
      "type": "System.Type",
      "address": 96
    },
    "__0_get_logPrefix__ret": {
      "name": "__0_get_logPrefix__ret",
      "type": "System.String",
      "address": 477
    },
    "__intnl_VRCUdonUdonBehaviour_7": {
      "name": "__intnl_VRCUdonUdonBehaviour_7",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 694
    },
    "__19__intnlparam": {
      "name": "__19__intnlparam",
      "type": "System.Int32",
      "address": 387
    },
    "__intnl_UnityEngineGameObject_11": {
      "name": "__intnl_UnityEngineGameObject_11",
      "type": "UnityEngine.GameObject",
      "address": 739
    },
    "__0_inkIdVector__param": {
      "name": "__0_inkIdVector__param",
      "type": "UnityEngine.Vector3",
      "address": 319
    },
    "__lcl_modeAsInt_SystemInt32_1": {
      "name": "__lcl_modeAsInt_SystemInt32_1",
      "type": "System.Int32",
      "address": 778
    },
    "__lcl_modeAsInt_SystemInt32_0": {
      "name": "__lcl_modeAsInt_SystemInt32_0",
      "type": "System.Int32",
      "address": 756
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 133
    },
    "penIdString": {
      "name": "penIdString",
      "type": "System.String",
      "address": 44
    },
    "__const_UnityEngineKeyCode_1": {
      "name": "__const_UnityEngineKeyCode_1",
      "type": "UnityEngine.KeyCode",
      "address": 176
    },
    "prevClickTime": {
      "name": "prevClickTime",
      "type": "System.Single",
      "address": 37
    },
    "__const_SystemSingle_7": {
      "name": "__const_SystemSingle_7",
      "type": "System.Single",
      "address": 194
    },
    "__24__intnlparam": {
      "name": "__24__intnlparam",
      "type": "System.Boolean",
      "address": 399
    },
    "__const_VRCSDK3DataTokenType_0": {
      "name": "__const_VRCSDK3DataTokenType_0",
      "type": "VRC.SDK3.Data.TokenType",
      "address": 379
    },
    "__intnl_UnityEngineVector3_5": {
      "name": "__intnl_UnityEngineVector3_5",
      "type": "UnityEngine.Vector3",
      "address": 615
    },
    "__const_SystemInt32_19": {
      "name": "__const_SystemInt32_19",
      "type": "System.Int32",
      "address": 507
    },
    "useDoubleClick": {
      "name": "useDoubleClick",
      "type": "System.Boolean",
      "address": 36
    },
    "__gintnl_SystemUInt32_93": {
      "name": "__gintnl_SystemUInt32_93",
      "type": "System.UInt32",
      "address": 348
    },
    "__gintnl_SystemUInt32_83": {
      "name": "__gintnl_SystemUInt32_83",
      "type": "System.UInt32",
      "address": 326
    },
    "__gintnl_SystemUInt32_53": {
      "name": "__gintnl_SystemUInt32_53",
      "type": "System.UInt32",
      "address": 282
    },
    "__gintnl_SystemUInt32_43": {
      "name": "__gintnl_SystemUInt32_43",
      "type": "System.UInt32",
      "address": 257
    },
    "__gintnl_SystemUInt32_73": {
      "name": "__gintnl_SystemUInt32_73",
      "type": "System.UInt32",
      "address": 304
    },
    "__gintnl_SystemUInt32_63": {
      "name": "__gintnl_SystemUInt32_63",
      "type": "System.UInt32",
      "address": 293
    },
    "__gintnl_SystemUInt32_13": {
      "name": "__gintnl_SystemUInt32_13",
      "type": "System.UInt32",
      "address": 129
    },
    "__gintnl_SystemUInt32_33": {
      "name": "__gintnl_SystemUInt32_33",
      "type": "System.UInt32",
      "address": 212
    },
    "__gintnl_SystemUInt32_23": {
      "name": "__gintnl_SystemUInt32_23",
      "type": "System.UInt32",
      "address": 175
    },
    "__intnl_UnityEngineCollider_1": {
      "name": "__intnl_UnityEngineCollider_1",
      "type": "UnityEngine.Collider",
      "address": 891
    },
    "__intnl_UnityEngineCollider_0": {
      "name": "__intnl_UnityEngineCollider_0",
      "type": "UnityEngine.Collider",
      "address": 654
    },
    "__3__intnlparam": {
      "name": "__3__intnlparam",
      "type": "UnityEngine.Transform",
      "address": 120
    },
    "__intnl_UnityEngineObject_9": {
      "name": "__intnl_UnityEngineObject_9",
      "type": "UnityEngine.Object",
      "address": 902
    },
    "__intnl_UnityEngineObject_8": {
      "name": "__intnl_UnityEngineObject_8",
      "type": "UnityEngine.Object",
      "address": 900
    },
    "__intnl_UnityEngineObject_7": {
      "name": "__intnl_UnityEngineObject_7",
      "type": "UnityEngine.Object",
      "address": 844
    },
    "__intnl_UnityEngineObject_6": {
      "name": "__intnl_UnityEngineObject_6",
      "type": "UnityEngine.Object",
      "address": 719
    },
    "__intnl_UnityEngineObject_5": {
      "name": "__intnl_UnityEngineObject_5",
      "type": "UnityEngine.Object",
      "address": 662
    },
    "__intnl_UnityEngineObject_4": {
      "name": "__intnl_UnityEngineObject_4",
      "type": "UnityEngine.Object",
      "address": 661
    },
    "__intnl_UnityEngineObject_3": {
      "name": "__intnl_UnityEngineObject_3",
      "type": "UnityEngine.Object",
      "address": 659
    },
    "__intnl_UnityEngineObject_2": {
      "name": "__intnl_UnityEngineObject_2",
      "type": "UnityEngine.Object",
      "address": 658
    },
    "__intnl_UnityEngineObject_1": {
      "name": "__intnl_UnityEngineObject_1",
      "type": "UnityEngine.Object",
      "address": 588
    },
    "__intnl_UnityEngineObject_0": {
      "name": "__intnl_UnityEngineObject_0",
      "type": "UnityEngine.Object",
      "address": 585
    },
    "__const_SystemInt32_16": {
      "name": "__const_SystemInt32_16",
      "type": "System.Int32",
      "address": 497
    },
    "inkColliderLayer": {
      "name": "inkColliderLayer",
      "type": "System.Int32",
      "address": 32
    },
    "useSurftraceMode": {
      "name": "useSurftraceMode",
      "type": "System.Boolean",
      "address": 69
    },
    "__lcl_penId_SystemInt32_2": {
      "name": "__lcl_penId_SystemInt32_2",
      "type": "System.Int32",
      "address": 880
    },
    "__lcl_penId_SystemInt32_0": {
      "name": "__lcl_penId_SystemInt32_0",
      "type": "System.Int32",
      "address": 811
    },
    "__lcl_penId_SystemInt32_1": {
      "name": "__lcl_penId_SystemInt32_1",
      "type": "System.Int32",
      "address": 872
    },
    "__const_VRCUdonCommonInterfacesNetworkEventTarget_0": {
      "name": "__const_VRCUdonCommonInterfacesNetworkEventTarget_0",
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "address": 180
    },
    "__0_get_pickup__ret": {
      "name": "__0_get_pickup__ret",
      "type": "VRC.SDK3.Components.VRCPickup",
      "address": 94
    },
    "__lcl_inkIdVector_UnityEngineVector3_1": {
      "name": "__lcl_inkIdVector_UnityEngineVector3_1",
      "type": "UnityEngine.Vector3",
      "address": 810
    },
    "__intnl_SystemString_6": {
      "name": "__intnl_SystemString_6",
      "type": "System.String",
      "address": 711
    },
    "__16__intnlparam": {
      "name": "__16__intnlparam",
      "type": "UnityEngine.Vector3[]",
      "address": 364
    },
    "__this_VRCUdonUdonBehaviour_17": {
      "name": "__this_VRCUdonUdonBehaviour_17",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 478
    },
    "__38__intnlparam": {
      "name": "__38__intnlparam",
      "type": "UnityEngine.GameObject",
      "address": 513
    },
    "__const_SystemString_16": {
      "name": "__const_SystemString_16",
      "type": "System.String",
      "address": 151
    },
    "__const_SystemString_17": {
      "name": "__const_SystemString_17",
      "type": "System.String",
      "address": 158
    },
    "__const_SystemString_14": {
      "name": "__const_SystemString_14",
      "type": "System.String",
      "address": 147
    },
    "__const_SystemString_15": {
      "name": "__const_SystemString_15",
      "type": "System.String",
      "address": 149
    },
    "__const_SystemString_12": {
      "name": "__const_SystemString_12",
      "type": "System.String",
      "address": 145
    },
    "__const_SystemString_13": {
      "name": "__const_SystemString_13",
      "type": "System.String",
      "address": 146
    },
    "__const_SystemString_10": {
      "name": "__const_SystemString_10",
      "type": "System.String",
      "address": 143
    },
    "__const_SystemString_11": {
      "name": "__const_SystemString_11",
      "type": "System.String",
      "address": 144
    },
    "__const_SystemString_18": {
      "name": "__const_SystemString_18",
      "type": "System.String",
      "address": 159
    },
    "__const_SystemString_19": {
      "name": "__const_SystemString_19",
      "type": "System.String",
      "address": 160
    },
    "__intnl_SystemSingle_7": {
      "name": "__intnl_SystemSingle_7",
      "type": "System.Single",
      "address": 541
    },
    "isUser": {
      "name": "isUser",
      "type": "System.Boolean",
      "address": 25
    },
    "__gintnl_SystemUInt32_138": {
      "name": "__gintnl_SystemUInt32_138",
      "type": "System.UInt32",
      "address": 447
    },
    "__gintnl_SystemUInt32_139": {
      "name": "__gintnl_SystemUInt32_139",
      "type": "System.UInt32",
      "address": 448
    },
    "__gintnl_SystemUInt32_134": {
      "name": "__gintnl_SystemUInt32_134",
      "type": "System.UInt32",
      "address": 442
    },
    "__gintnl_SystemUInt32_135": {
      "name": "__gintnl_SystemUInt32_135",
      "type": "System.UInt32",
      "address": 443
    },
    "__gintnl_SystemUInt32_136": {
      "name": "__gintnl_SystemUInt32_136",
      "type": "System.UInt32",
      "address": 445
    },
    "__gintnl_SystemUInt32_137": {
      "name": "__gintnl_SystemUInt32_137",
      "type": "System.UInt32",
      "address": 446
    },
    "__gintnl_SystemUInt32_130": {
      "name": "__gintnl_SystemUInt32_130",
      "type": "System.UInt32",
      "address": 433
    },
    "__gintnl_SystemUInt32_131": {
      "name": "__gintnl_SystemUInt32_131",
      "type": "System.UInt32",
      "address": 434
    },
    "__gintnl_SystemUInt32_132": {
      "name": "__gintnl_SystemUInt32_132",
      "type": "System.UInt32",
      "address": 440
    },
    "__gintnl_SystemUInt32_133": {
      "name": "__gintnl_SystemUInt32_133",
      "type": "System.UInt32",
      "address": 441
    },
    "__0_get_isHeld__ret": {
      "name": "__0_get_isHeld__ret",
      "type": "System.Boolean",
      "address": 202
    },
    "__intnl_VRCSDKBaseVRCPlayerApi_0": {
      "name": "__intnl_VRCSDKBaseVRCPlayerApi_0",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 733
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__intnl_SystemObject_7": {
      "name": "__intnl_SystemObject_7",
      "type": "System.Object",
      "address": 577
    },
    "__intnl_UnityEngineShader_0": {
      "name": "__intnl_UnityEngineShader_0",
      "type": "UnityEngine.Shader",
      "address": 587
    },
    "__intnl_VRCUdonUdonBehaviour_2": {
      "name": "__intnl_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 558
    },
    "mouseDelta": {
      "name": "mouseDelta",
      "type": "UnityEngine.Vector2",
      "address": 63
    },
    "__0_get_localPlayerId__ret": {
      "name": "__0_get_localPlayerId__ret",
      "type": "System.Int32",
      "address": 104
    },
    "__lcl_mode_SystemInt32_0": {
      "name": "__lcl_mode_SystemInt32_0",
      "type": "System.Int32",
      "address": 790
    },
    "__intnl_UnityEngineGameObject_14": {
      "name": "__intnl_UnityEngineGameObject_14",
      "type": "UnityEngine.GameObject",
      "address": 777
    },
    "__intnl_UnityEngineTransform_5": {
      "name": "__intnl_UnityEngineTransform_5",
      "type": "UnityEngine.Transform",
      "address": 561
    },
    "__31__intnlparam": {
      "name": "__31__intnlparam",
      "type": "System.Boolean",
      "address": 435
    },
    "__0___0_TryGetLastLocalInk__ret": {
      "name": "__0___0_TryGetLastLocalInk__ret",
      "type": "System.Boolean",
      "address": 377
    },
    "__lcl_sphereCollider_UnityEngineSphereCollider_0": {
      "name": "__lcl_sphereCollider_UnityEngineSphereCollider_0",
      "type": "UnityEngine.SphereCollider",
      "address": 515
    },
    "__intnl_VRCUdonUdonBehaviour_9": {
      "name": "__intnl_VRCUdonUdonBehaviour_9",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 748
    },
    "__intnl_SystemDouble_4": {
      "name": "__intnl_SystemDouble_4",
      "type": "System.Double",
      "address": 547
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 77
    },
    "__lcl_data_UnityEngineVector3Array_0": {
      "name": "__lcl_data_UnityEngineVector3Array_0",
      "type": "UnityEngine.Vector3[]",
      "address": 746
    },
    "__intnl_UnityEngineComponent_0": {
      "name": "__intnl_UnityEngineComponent_0",
      "type": "UnityEngine.Component",
      "address": 527
    },
    "__intnl_SystemBoolean_81": {
      "name": "__intnl_SystemBoolean_81",
      "type": "System.Boolean",
      "address": 836
    },
    "__intnl_SystemBoolean_91": {
      "name": "__intnl_SystemBoolean_91",
      "type": "System.Boolean",
      "address": 896
    },
    "__intnl_SystemBoolean_11": {
      "name": "__intnl_SystemBoolean_11",
      "type": "System.Boolean",
      "address": 592
    },
    "__intnl_SystemBoolean_21": {
      "name": "__intnl_SystemBoolean_21",
      "type": "System.Boolean",
      "address": 613
    },
    "__intnl_SystemBoolean_31": {
      "name": "__intnl_SystemBoolean_31",
      "type": "System.Boolean",
      "address": 669
    },
    "__intnl_SystemBoolean_41": {
      "name": "__intnl_SystemBoolean_41",
      "type": "System.Boolean",
      "address": 701
    },
    "__intnl_SystemBoolean_51": {
      "name": "__intnl_SystemBoolean_51",
      "type": "System.Boolean",
      "address": 722
    },
    "__intnl_SystemBoolean_61": {
      "name": "__intnl_SystemBoolean_61",
      "type": "System.Boolean",
      "address": 732
    },
    "__intnl_SystemBoolean_71": {
      "name": "__intnl_SystemBoolean_71",
      "type": "System.Boolean",
      "address": 791
    },
    "__const_UnityEngineQueryTriggerInteraction_0": {
      "name": "__const_UnityEngineQueryTriggerInteraction_0",
      "type": "UnityEngine.QueryTriggerInteraction",
      "address": 215
    },
    "__intnl_UnityEngineTransform_19": {
      "name": "__intnl_UnityEngineTransform_19",
      "type": "UnityEngine.Transform",
      "address": 775
    },
    "__intnl_UnityEngineTransform_18": {
      "name": "__intnl_UnityEngineTransform_18",
      "type": "UnityEngine.Transform",
      "address": 686
    },
    "__intnl_UnityEngineTransform_13": {
      "name": "__intnl_UnityEngineTransform_13",
      "type": "UnityEngine.Transform",
      "address": 644
    },
    "__intnl_UnityEngineTransform_12": {
      "name": "__intnl_UnityEngineTransform_12",
      "type": "UnityEngine.Transform",
      "address": 638
    },
    "__intnl_UnityEngineTransform_11": {
      "name": "__intnl_UnityEngineTransform_11",
      "type": "UnityEngine.Transform",
      "address": 637
    },
    "__intnl_UnityEngineTransform_10": {
      "name": "__intnl_UnityEngineTransform_10",
      "type": "UnityEngine.Transform",
      "address": 636
    },
    "__intnl_UnityEngineTransform_17": {
      "name": "__intnl_UnityEngineTransform_17",
      "type": "UnityEngine.Transform",
      "address": 664
    },
    "__intnl_UnityEngineTransform_16": {
      "name": "__intnl_UnityEngineTransform_16",
      "type": "UnityEngine.Transform",
      "address": 660
    },
    "__intnl_UnityEngineTransform_15": {
      "name": "__intnl_UnityEngineTransform_15",
      "type": "UnityEngine.Transform",
      "address": 657
    },
    "__intnl_UnityEngineTransform_14": {
      "name": "__intnl_UnityEngineTransform_14",
      "type": "UnityEngine.Transform",
      "address": 653
    },
    "__lcl_lineRenderer_UnityEngineLineRenderer_0": {
      "name": "__lcl_lineRenderer_UnityEngineLineRenderer_0",
      "type": "UnityEngine.LineRenderer",
      "address": 663
    },
    "__gintnl_SystemUInt32_94": {
      "name": "__gintnl_SystemUInt32_94",
      "type": "System.UInt32",
      "address": 349
    },
    "__gintnl_SystemUInt32_84": {
      "name": "__gintnl_SystemUInt32_84",
      "type": "System.UInt32",
      "address": 328
    },
    "__gintnl_SystemUInt32_54": {
      "name": "__gintnl_SystemUInt32_54",
      "type": "System.UInt32",
      "address": 284
    },
    "__gintnl_SystemUInt32_44": {
      "name": "__gintnl_SystemUInt32_44",
      "type": "System.UInt32",
      "address": 259
    },
    "__gintnl_SystemUInt32_74": {
      "name": "__gintnl_SystemUInt32_74",
      "type": "System.UInt32",
      "address": 305
    },
    "__gintnl_SystemUInt32_64": {
      "name": "__gintnl_SystemUInt32_64",
      "type": "System.UInt32",
      "address": 294
    },
    "__gintnl_SystemUInt32_14": {
      "name": "__gintnl_SystemUInt32_14",
      "type": "System.UInt32",
      "address": 130
    },
    "__gintnl_SystemUInt32_34": {
      "name": "__gintnl_SystemUInt32_34",
      "type": "System.UInt32",
      "address": 214
    },
    "__gintnl_SystemUInt32_24": {
      "name": "__gintnl_SystemUInt32_24",
      "type": "System.UInt32",
      "address": 177
    },
    "__intnl_SystemSingle_40": {
      "name": "__intnl_SystemSingle_40",
      "type": "System.Single",
      "address": 968
    },
    "__intnl_SystemSingle_41": {
      "name": "__intnl_SystemSingle_41",
      "type": "System.Single",
      "address": 975
    },
    "__intnl_SystemSingle_42": {
      "name": "__intnl_SystemSingle_42",
      "type": "System.Single",
      "address": 989
    },
    "__intnl_SystemSingle_43": {
      "name": "__intnl_SystemSingle_43",
      "type": "System.Single",
      "address": 1001
    },
    "__intnl_SystemSingle_44": {
      "name": "__intnl_SystemSingle_44",
      "type": "System.Single",
      "address": 1003
    },
    "__intnl_SystemSingle_45": {
      "name": "__intnl_SystemSingle_45",
      "type": "System.Single",
      "address": 1005
    },
    "__const_UnityEngineVector3_0": {
      "name": "__const_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 154
    },
    "__intnl_SystemInt32_17": {
      "name": "__intnl_SystemInt32_17",
      "type": "System.Int32",
      "address": 801
    },
    "__intnl_SystemInt32_37": {
      "name": "__intnl_SystemInt32_37",
      "type": "System.Int32",
      "address": 930
    },
    "__intnl_SystemInt32_27": {
      "name": "__intnl_SystemInt32_27",
      "type": "System.Int32",
      "address": 893
    },
    "__intnl_SystemInt32_57": {
      "name": "__intnl_SystemInt32_57",
      "type": "System.Int32",
      "address": 982
    },
    "__intnl_SystemInt32_47": {
      "name": "__intnl_SystemInt32_47",
      "type": "System.Int32",
      "address": 961
    },
    "_isCheckedPointerRadius": {
      "name": "_isCheckedPointerRadius",
      "type": "System.Boolean",
      "address": 15
    },
    "__lcl__discard_UnityEngineVector3_0": {
      "name": "__lcl__discard_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 861
    },
    "__0_penId__param": {
      "name": "__0_penId__param",
      "type": "System.Int32",
      "address": 429
    },
    "__intnl_SystemString_3": {
      "name": "__intnl_SystemString_3",
      "type": "System.String",
      "address": 603
    },
    "__0__intnlparam": {
      "name": "__0__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 109
    },
    "__this_VRCUdonUdonBehaviour_14": {
      "name": "__this_VRCUdonUdonBehaviour_14",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 263
    },
    "__const_SystemType_7": {
      "name": "__const_SystemType_7",
      "type": "System.Type",
      "address": 473
    },
    "__const_SystemString_66": {
      "name": "__const_SystemString_66",
      "type": "System.String",
      "address": 408
    },
    "__const_SystemString_67": {
      "name": "__const_SystemString_67",
      "type": "System.String",
      "address": 409
    },
    "__const_SystemString_64": {
      "name": "__const_SystemString_64",
      "type": "System.String",
      "address": 406
    },
    "__const_SystemString_65": {
      "name": "__const_SystemString_65",
      "type": "System.String",
      "address": 407
    },
    "__const_SystemString_62": {
      "name": "__const_SystemString_62",
      "type": "System.String",
      "address": 383
    },
    "__const_SystemString_63": {
      "name": "__const_SystemString_63",
      "type": "System.String",
      "address": 397
    },
    "__const_SystemString_60": {
      "name": "__const_SystemString_60",
      "type": "System.String",
      "address": 381
    },
    "__const_SystemString_61": {
      "name": "__const_SystemString_61",
      "type": "System.String",
      "address": 382
    },
    "__const_SystemString_68": {
      "name": "__const_SystemString_68",
      "type": "System.String",
      "address": 410
    },
    "__const_SystemString_69": {
      "name": "__const_SystemString_69",
      "type": "System.String",
      "address": 418
    },
    "__intnl_SystemSingle_2": {
      "name": "__intnl_SystemSingle_2",
      "type": "System.Single",
      "address": 520
    },
    "__gintnl_SystemUInt32_148": {
      "name": "__gintnl_SystemUInt32_148",
      "type": "System.UInt32",
      "address": 457
    },
    "__gintnl_SystemUInt32_149": {
      "name": "__gintnl_SystemUInt32_149",
      "type": "System.UInt32",
      "address": 462
    },
    "__gintnl_SystemUInt32_144": {
      "name": "__gintnl_SystemUInt32_144",
      "type": "System.UInt32",
      "address": 453
    },
    "__gintnl_SystemUInt32_145": {
      "name": "__gintnl_SystemUInt32_145",
      "type": "System.UInt32",
      "address": 454
    },
    "__gintnl_SystemUInt32_146": {
      "name": "__gintnl_SystemUInt32_146",
      "type": "System.UInt32",
      "address": 455
    },
    "__gintnl_SystemUInt32_147": {
      "name": "__gintnl_SystemUInt32_147",
      "type": "System.UInt32",
      "address": 456
    },
    "__gintnl_SystemUInt32_140": {
      "name": "__gintnl_SystemUInt32_140",
      "type": "System.UInt32",
      "address": 449
    },
    "__gintnl_SystemUInt32_141": {
      "name": "__gintnl_SystemUInt32_141",
      "type": "System.UInt32",
      "address": 450
    },
    "__gintnl_SystemUInt32_142": {
      "name": "__gintnl_SystemUInt32_142",
      "type": "System.UInt32",
      "address": 451
    },
    "__gintnl_SystemUInt32_143": {
      "name": "__gintnl_SystemUInt32_143",
      "type": "System.UInt32",
      "address": 452
    },
    "__7__intnlparam": {
      "name": "__7__intnlparam",
      "type": "UnityEngine.Component",
      "address": 137
    },
    "_isCheckedLocalPlayerIdVector": {
      "name": "_isCheckedLocalPlayerIdVector",
      "type": "System.Boolean",
      "address": 51
    },
    "_logPrefix": {
      "name": "_logPrefix",
      "type": "System.String",
      "address": 75
    },
    "__intnl_SystemObject_2": {
      "name": "__intnl_SystemObject_2",
      "type": "System.Object",
      "address": 560
    },
    "__9__intnlparam": {
      "name": "__9__intnlparam",
      "type": "System.Int32",
      "address": 250
    },
    "__23__intnlparam": {
      "name": "__23__intnlparam",
      "type": "UnityEngine.GameObject",
      "address": 396
    },
    "__2_mode__param": {
      "name": "__2_mode__param",
      "type": "System.Int32",
      "address": 345
    },
    "__intnl_SystemObject_9": {
      "name": "__intnl_SystemObject_9",
      "type": "System.Object",
      "address": 581
    },
    "__0_lineRenderer__param": {
      "name": "__0_lineRenderer__param",
      "type": "UnityEngine.LineRenderer",
      "address": 344
    },
    "__0_get_inkPrefabCollider__ret": {
      "name": "__0_get_inkPrefabCollider__ret",
      "type": "UnityEngine.MeshCollider",
      "address": 91
    },
    "__lcl_penIdVector_UnityEngineVector3_1": {
      "name": "__lcl_penIdVector_UnityEngineVector3_1",
      "type": "UnityEngine.Vector3",
      "address": 859
    },
    "__intnl_VRCUdonUdonBehaviour_1": {
      "name": "__intnl_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 555
    },
    "__lcl_t2_UnityEngineTransform_0": {
      "name": "__lcl_t2_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 649
    },
    "inkMeshLayer": {
      "name": "inkMeshLayer",
      "type": "System.Int32",
      "address": 31
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 127
    },
    "pointer": {
      "name": "pointer",
      "type": "UnityEngine.Transform",
      "address": 14
    },
    "__intnl_UnityEngineGameObject_17": {
      "name": "__intnl_UnityEngineGameObject_17",
      "type": "UnityEngine.GameObject",
      "address": 817
    },
    "__intnl_UnityEngineTransform_2": {
      "name": "__intnl_UnityEngineTransform_2",
      "type": "UnityEngine.Transform",
      "address": 553
    },
    "__lcl_count_SystemInt32_0": {
      "name": "__lcl_count_SystemInt32_0",
      "type": "System.Int32",
      "address": 642
    },
    "__lcl_count_SystemInt32_1": {
      "name": "__lcl_count_SystemInt32_1",
      "type": "System.Int32",
      "address": 883
    },
    "__const_SystemSingle_2": {
      "name": "__const_SystemSingle_2",
      "type": "System.Single",
      "address": 156
    },
    "__intnl_SystemDouble_1": {
      "name": "__intnl_SystemDouble_1",
      "type": "System.Double",
      "address": 540
    },
    "__intnl_SystemBoolean_89": {
      "name": "__intnl_SystemBoolean_89",
      "type": "System.Boolean",
      "address": 890
    },
    "__intnl_SystemBoolean_99": {
      "name": "__intnl_SystemBoolean_99",
      "type": "System.Boolean",
      "address": 940
    },
    "__intnl_SystemBoolean_19": {
      "name": "__intnl_SystemBoolean_19",
      "type": "System.Boolean",
      "address": 600
    },
    "__intnl_SystemBoolean_29": {
      "name": "__intnl_SystemBoolean_29",
      "type": "System.Boolean",
      "address": 665
    },
    "__intnl_SystemBoolean_39": {
      "name": "__intnl_SystemBoolean_39",
      "type": "System.Boolean",
      "address": 695
    },
    "__intnl_SystemBoolean_49": {
      "name": "__intnl_SystemBoolean_49",
      "type": "System.Boolean",
      "address": 720
    },
    "__intnl_SystemBoolean_59": {
      "name": "__intnl_SystemBoolean_59",
      "type": "System.Boolean",
      "address": 730
    },
    "__intnl_SystemBoolean_69": {
      "name": "__intnl_SystemBoolean_69",
      "type": "System.Boolean",
      "address": 786
    },
    "__intnl_SystemBoolean_79": {
      "name": "__intnl_SystemBoolean_79",
      "type": "System.Boolean",
      "address": 807
    },
    "__37__intnlparam": {
      "name": "__37__intnlparam",
      "type": "UnityEngine.Color",
      "address": 488
    },
    "__intnl_UnityEngineVector3_0": {
      "name": "__intnl_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 564
    },
    "__intnl_UnityEngineGameObject_9": {
      "name": "__intnl_UnityEngineGameObject_9",
      "type": "UnityEngine.GameObject",
      "address": 716
    },
    "__intnl_UnityEngineGameObject_8": {
      "name": "__intnl_UnityEngineGameObject_8",
      "type": "UnityEngine.GameObject",
      "address": 714
    },
    "__intnl_UnityEngineGameObject_3": {
      "name": "__intnl_UnityEngineGameObject_3",
      "type": "UnityEngine.GameObject",
      "address": 573
    },
    "__intnl_UnityEngineGameObject_2": {
      "name": "__intnl_UnityEngineGameObject_2",
      "type": "UnityEngine.GameObject",
      "address": 572
    },
    "__intnl_UnityEngineGameObject_1": {
      "name": "__intnl_UnityEngineGameObject_1",
      "type": "UnityEngine.GameObject",
      "address": 571
    },
    "__intnl_UnityEngineGameObject_0": {
      "name": "__intnl_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 562
    },
    "__intnl_UnityEngineGameObject_7": {
      "name": "__intnl_UnityEngineGameObject_7",
      "type": "UnityEngine.GameObject",
      "address": 672
    },
    "__intnl_UnityEngineGameObject_6": {
      "name": "__intnl_UnityEngineGameObject_6",
      "type": "UnityEngine.GameObject",
      "address": 667
    },
    "__intnl_UnityEngineGameObject_5": {
      "name": "__intnl_UnityEngineGameObject_5",
      "type": "UnityEngine.GameObject",
      "address": 608
    },
    "__intnl_UnityEngineGameObject_4": {
      "name": "__intnl_UnityEngineGameObject_4",
      "type": "UnityEngine.GameObject",
      "address": 604
    },
    "__lcl_data_UnityEngineVector3Array_3": {
      "name": "__lcl_data_UnityEngineVector3Array_3",
      "type": "UnityEngine.Vector3[]",
      "address": 853
    },
    "__const_SystemSingle_9": {
      "name": "__const_SystemSingle_9",
      "type": "System.Single",
      "address": 213
    },
    "__intnl_UnityEngineVector2_1": {
      "name": "__intnl_UnityEngineVector2_1",
      "type": "UnityEngine.Vector2",
      "address": 625
    },
    "__lcl_t1_UnityEngineTransform_0": {
      "name": "__lcl_t1_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 648
    },
    "__intnl_SystemBoolean_84": {
      "name": "__intnl_SystemBoolean_84",
      "type": "System.Boolean",
      "address": 869
    },
    "__intnl_SystemBoolean_94": {
      "name": "__intnl_SystemBoolean_94",
      "type": "System.Boolean",
      "address": 906
    },
    "__intnl_SystemBoolean_14": {
      "name": "__intnl_SystemBoolean_14",
      "type": "System.Boolean",
      "address": 595
    },
    "__intnl_SystemBoolean_24": {
      "name": "__intnl_SystemBoolean_24",
      "type": "System.Boolean",
      "address": 646
    },
    "__intnl_SystemBoolean_34": {
      "name": "__intnl_SystemBoolean_34",
      "type": "System.Boolean",
      "address": 676
    },
    "__intnl_SystemBoolean_44": {
      "name": "__intnl_SystemBoolean_44",
      "type": "System.Boolean",
      "address": 707
    },
    "__intnl_SystemBoolean_54": {
      "name": "__intnl_SystemBoolean_54",
      "type": "System.Boolean",
      "address": 725
    },
    "__intnl_SystemBoolean_64": {
      "name": "__intnl_SystemBoolean_64",
      "type": "System.Boolean",
      "address": 736
    },
    "__intnl_SystemBoolean_74": {
      "name": "__intnl_SystemBoolean_74",
      "type": "System.Boolean",
      "address": 794
    },
    "center": {
      "name": "center",
      "type": "UnityEngine.Vector3",
      "address": 58
    },
    "__lcl_material_UnityEngineMaterial_0": {
      "name": "__lcl_material_UnityEngineMaterial_0",
      "type": "UnityEngine.Material",
      "address": 574
    },
    "__0___0__CheckId__ret": {
      "name": "__0___0__CheckId__ret",
      "type": "System.Boolean",
      "address": 170
    },
    "__const_SystemInt32_13": {
      "name": "__const_SystemInt32_13",
      "type": "System.Int32",
      "address": 471
    },
    "localInkHistory": {
      "name": "localInkHistory",
      "type": "VRC.SDK3.Data.DataList",
      "address": 55
    },
    "__intnl_UnityEngineComponentArray_0": {
      "name": "__intnl_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 927
    },
    "__intnl_SystemSingle_30": {
      "name": "__intnl_SystemSingle_30",
      "type": "System.Single",
      "address": 863
    },
    "__intnl_SystemSingle_31": {
      "name": "__intnl_SystemSingle_31",
      "type": "System.Single",
      "address": 864
    },
    "__intnl_SystemSingle_32": {
      "name": "__intnl_SystemSingle_32",
      "type": "System.Single",
      "address": 911
    },
    "__intnl_SystemSingle_33": {
      "name": "__intnl_SystemSingle_33",
      "type": "System.Single",
      "address": 913
    },
    "__intnl_SystemSingle_34": {
      "name": "__intnl_SystemSingle_34",
      "type": "System.Single",
      "address": 915
    },
    "__intnl_SystemSingle_35": {
      "name": "__intnl_SystemSingle_35",
      "type": "System.Single",
      "address": 920
    },
    "__intnl_SystemSingle_36": {
      "name": "__intnl_SystemSingle_36",
      "type": "System.Single",
      "address": 923
    },
    "__intnl_SystemSingle_37": {
      "name": "__intnl_SystemSingle_37",
      "type": "System.Single",
      "address": 925
    },
    "__intnl_SystemSingle_38": {
      "name": "__intnl_SystemSingle_38",
      "type": "System.Single",
      "address": 953
    },
    "__intnl_SystemSingle_39": {
      "name": "__intnl_SystemSingle_39",
      "type": "System.Single",
      "address": 962
    },
    "__intnl_SystemInt32_14": {
      "name": "__intnl_SystemInt32_14",
      "type": "System.Int32",
      "address": 779
    },
    "__intnl_SystemInt32_34": {
      "name": "__intnl_SystemInt32_34",
      "type": "System.Int32",
      "address": 922
    },
    "__intnl_SystemInt32_24": {
      "name": "__intnl_SystemInt32_24",
      "type": "System.Int32",
      "address": 867
    },
    "__intnl_SystemInt32_54": {
      "name": "__intnl_SystemInt32_54",
      "type": "System.Int32",
      "address": 974
    },
    "__intnl_SystemInt32_44": {
      "name": "__intnl_SystemInt32_44",
      "type": "System.Int32",
      "address": 958
    },
    "__intnl_SystemInt32_64": {
      "name": "__intnl_SystemInt32_64",
      "type": "System.Int32",
      "address": 1006
    },
    "__intnl_SystemInt32_19": {
      "name": "__intnl_SystemInt32_19",
      "type": "System.Int32",
      "address": 806
    },
    "__intnl_SystemInt32_39": {
      "name": "__intnl_SystemInt32_39",
      "type": "System.Int32",
      "address": 948
    },
    "__intnl_SystemInt32_29": {
      "name": "__intnl_SystemInt32_29",
      "type": "System.Int32",
      "address": 912
    },
    "__intnl_SystemInt32_59": {
      "name": "__intnl_SystemInt32_59",
      "type": "System.Int32",
      "address": 990
    },
    "__intnl_SystemInt32_49": {
      "name": "__intnl_SystemInt32_49",
      "type": "System.Int32",
      "address": 966
    },
    "__const_SystemObject_0": {
      "name": "__const_SystemObject_0",
      "type": "System.Object",
      "address": 103
    },
    "__gintnl_SystemSingleArray_0": {
      "name": "__gintnl_SystemSingleArray_0",
      "type": "System.Single[]",
      "address": 82
    },
    "__0_targetMode__param": {
      "name": "__0_targetMode__param",
      "type": "System.Int32",
      "address": 361
    },
    "__intnl_UnityEngineGradient_1": {
      "name": "__intnl_UnityEngineGradient_1",
      "type": "UnityEngine.Gradient",
      "address": 580
    },
    "__intnl_UnityEngineGradient_0": {
      "name": "__intnl_UnityEngineGradient_0",
      "type": "UnityEngine.Gradient",
      "address": 578
    },
    "__lcl_mesh_UnityEngineMesh_0": {
      "name": "__lcl_mesh_UnityEngineMesh_0",
      "type": "UnityEngine.Mesh",
      "address": 848
    },
    "__15__intnlparam": {
      "name": "__15__intnlparam",
      "type": "System.Int32",
      "address": 363
    },
    "__intnl_SystemString_0": {
      "name": "__intnl_SystemString_0",
      "type": "System.String",
      "address": 533
    },
    "__intnl_UnityEngineVector3_20": {
      "name": "__intnl_UnityEngineVector3_20",
      "type": "UnityEngine.Vector3",
      "address": 702
    },
    "__intnl_UnityEngineVector3_21": {
      "name": "__intnl_UnityEngineVector3_21",
      "type": "UnityEngine.Vector3",
      "address": 760
    },
    "__intnl_UnityEngineVector3_22": {
      "name": "__intnl_UnityEngineVector3_22",
      "type": "UnityEngine.Vector3",
      "address": 764
    },
    "__intnl_UnityEngineVector3_23": {
      "name": "__intnl_UnityEngineVector3_23",
      "type": "UnityEngine.Vector3",
      "address": 781
    },
    "__intnl_UnityEngineVector3_24": {
      "name": "__intnl_UnityEngineVector3_24",
      "type": "UnityEngine.Vector3",
      "address": 784
    },
    "__intnl_UnityEngineVector3_25": {
      "name": "__intnl_UnityEngineVector3_25",
      "type": "UnityEngine.Vector3",
      "address": 856
    },
    "__intnl_UnityEngineVector3_26": {
      "name": "__intnl_UnityEngineVector3_26",
      "type": "UnityEngine.Vector3",
      "address": 857
    },
    "__intnl_UnityEngineVector3_27": {
      "name": "__intnl_UnityEngineVector3_27",
      "type": "UnityEngine.Vector3",
      "address": 865
    },
    "__intnl_UnityEngineVector3_28": {
      "name": "__intnl_UnityEngineVector3_28",
      "type": "UnityEngine.Vector3",
      "address": 884
    },
    "__intnl_UnityEngineVector3_29": {
      "name": "__intnl_UnityEngineVector3_29",
      "type": "UnityEngine.Vector3",
      "address": 916
    },
    "_inkPrefabCollider": {
      "name": "_inkPrefabCollider",
      "type": "UnityEngine.MeshCollider",
      "address": 24
    },
    "__intnl_SystemType_1": {
      "name": "__intnl_SystemType_1",
      "type": "System.Type",
      "address": 933
    },
    "results32": {
      "name": "results32",
      "type": "UnityEngine.Collider[]",
      "address": 73
    },
    "__lcl_positions_UnityEngineVector3Array_1": {
      "name": "__lcl_positions_UnityEngineVector3Array_1",
      "type": "UnityEngine.Vector3[]",
      "address": 768
    },
    "__lcl_positions_UnityEngineVector3Array_0": {
      "name": "__lcl_positions_UnityEngineVector3Array_0",
      "type": "UnityEngine.Vector3[]",
      "address": 752
    },
    "__this_VRCUdonUdonBehaviour_19": {
      "name": "__this_VRCUdonUdonBehaviour_19",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 482
    },
    "__const_SystemType_0": {
      "name": "__const_SystemType_0",
      "type": "System.Type",
      "address": 79
    },
    "__intnl_UnityEngineRect_0": {
      "name": "__intnl_UnityEngineRect_0",
      "type": "UnityEngine.Rect",
      "address": 607
    },
    "__intnl_SystemSingle_1": {
      "name": "__intnl_SystemSingle_1",
      "type": "System.Single",
      "address": 519
    },
    "__0_target__param": {
      "name": "__0_target__param",
      "type": "UnityEngine.Collider",
      "address": 222
    },
    "__4_data__param": {
      "name": "__4_data__param",
      "type": "UnityEngine.Vector3[]",
      "address": 327
    },
    "__intnl_SystemObject_1": {
      "name": "__intnl_SystemObject_1",
      "type": "System.Object",
      "address": 557
    },
    "__20__intnlparam": {
      "name": "__20__intnlparam",
      "type": "System.Int32",
      "address": 390
    },
    "__0_inkId__param": {
      "name": "__0_inkId__param",
      "type": "System.Int32",
      "address": 324
    },
    "__lcl_inkCollider_UnityEngineMeshCollider_0": {
      "name": "__lcl_inkCollider_UnityEngineMeshCollider_0",
      "type": "UnityEngine.MeshCollider",
      "address": 846
    },
    "__1_mode__param": {
      "name": "__1_mode__param",
      "type": "System.Int32",
      "address": 318
    },
    "__intnl_SystemDouble_9": {
      "name": "__intnl_SystemDouble_9",
      "type": "System.Double",
      "address": 830
    },
    "__const_SystemString_7": {
      "name": "__const_SystemString_7",
      "type": "System.String",
      "address": 139
    },
    "__const_UnityEngineKeyCode_3": {
      "name": "__const_UnityEngineKeyCode_3",
      "type": "UnityEngine.KeyCode",
      "address": 182
    },
    "__4__intnlparam": {
      "name": "__4__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 125
    },
    "_penIdVector_k__BackingField": {
      "name": "_penIdVector_k__BackingField",
      "type": "UnityEngine.Vector3",
      "address": 43
    },
    "__intnl_UnityEngineVector3_8": {
      "name": "__intnl_UnityEngineVector3_8",
      "type": "UnityEngine.Vector3",
      "address": 618
    },
    "isPickedUp": {
      "name": "isPickedUp",
      "type": "System.Boolean",
      "address": 72
    },
    "__const_SystemSingle_1": {
      "name": "__const_SystemSingle_1",
      "type": "System.Single",
      "address": 90
    },
    "__intnl_SystemDouble_2": {
      "name": "__intnl_SystemDouble_2",
      "type": "System.Double",
      "address": 543
    },
    "__intnl_UnityEngineVector3_7": {
      "name": "__intnl_UnityEngineVector3_7",
      "type": "UnityEngine.Vector3",
      "address": 617
    },
    "__const_VRCSDKBaseVRCPlayerApiTrackingDataType_0": {
      "name": "__const_VRCSDKBaseVRCPlayerApiTrackingDataType_0",
      "type": "VRC.SDKBase.VRCPlayerApi+TrackingDataType",
      "address": 205
    },
    "__gintnl_SystemUInt32_91": {
      "name": "__gintnl_SystemUInt32_91",
      "type": "System.UInt32",
      "address": 340
    },
    "__gintnl_SystemUInt32_81": {
      "name": "__gintnl_SystemUInt32_81",
      "type": "System.UInt32",
      "address": 322
    },
    "__gintnl_SystemUInt32_51": {
      "name": "__gintnl_SystemUInt32_51",
      "type": "System.UInt32",
      "address": 279
    },
    "__gintnl_SystemUInt32_41": {
      "name": "__gintnl_SystemUInt32_41",
      "type": "System.UInt32",
      "address": 248
    },
    "__gintnl_SystemUInt32_71": {
      "name": "__gintnl_SystemUInt32_71",
      "type": "System.UInt32",
      "address": 301
    },
    "__gintnl_SystemUInt32_61": {
      "name": "__gintnl_SystemUInt32_61",
      "type": "System.UInt32",
      "address": 291
    },
    "__gintnl_SystemUInt32_11": {
      "name": "__gintnl_SystemUInt32_11",
      "type": "System.UInt32",
      "address": 124
    },
    "__gintnl_SystemUInt32_31": {
      "name": "__gintnl_SystemUInt32_31",
      "type": "System.UInt32",
      "address": 204
    },
    "__gintnl_SystemUInt32_21": {
      "name": "__gintnl_SystemUInt32_21",
      "type": "System.UInt32",
      "address": 172
    },
    "__3_value__param": {
      "name": "__3_value__param",
      "type": "System.Boolean",
      "address": 277
    },
    "__34__intnlparam": {
      "name": "__34__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 438
    },
    "ratio": {
      "name": "ratio",
      "type": "System.Single",
      "address": 64
    },
    "__intnl_SystemBoolean_87": {
      "name": "__intnl_SystemBoolean_87",
      "type": "System.Boolean",
      "address": 881
    },
    "__intnl_SystemBoolean_97": {
      "name": "__intnl_SystemBoolean_97",
      "type": "System.Boolean",
      "address": 935
    },
    "__intnl_SystemBoolean_17": {
      "name": "__intnl_SystemBoolean_17",
      "type": "System.Boolean",
      "address": 598
    },
    "__intnl_SystemBoolean_27": {
      "name": "__intnl_SystemBoolean_27",
      "type": "System.Boolean",
      "address": 655
    },
    "__intnl_SystemBoolean_37": {
      "name": "__intnl_SystemBoolean_37",
      "type": "System.Boolean",
      "address": 685
    },
    "__intnl_SystemBoolean_47": {
      "name": "__intnl_SystemBoolean_47",
      "type": "System.Boolean",
      "address": 715
    },
    "__intnl_SystemBoolean_57": {
      "name": "__intnl_SystemBoolean_57",
      "type": "System.Boolean",
      "address": 728
    },
    "__intnl_SystemBoolean_67": {
      "name": "__intnl_SystemBoolean_67",
      "type": "System.Boolean",
      "address": 765
    },
    "__intnl_SystemBoolean_77": {
      "name": "__intnl_SystemBoolean_77",
      "type": "System.Boolean",
      "address": 802
    },
    "__0_idVector__param": {
      "name": "__0_idVector__param",
      "type": "UnityEngine.Vector3",
      "address": 171
    },
    "__lcl_idHolder_UnityEngineTransform_1": {
      "name": "__lcl_idHolder_UnityEngineTransform_1",
      "type": "UnityEngine.Transform",
      "address": 998
    },
    "__lcl_idHolder_UnityEngineTransform_0": {
      "name": "__lcl_idHolder_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 984
    },
    "__const_SystemInt32_10": {
      "name": "__const_SystemInt32_10",
      "type": "System.Int32",
      "address": 367
    },
    "__const_SystemInt32_20": {
      "name": "__const_SystemInt32_20",
      "type": "System.Int32",
      "address": 509
    },
    "__12__intnlparam": {
      "name": "__12__intnlparam",
      "type": "UnityEngine.Vector3[]",
      "address": 332
    },
    "__0_penManager__param": {
      "name": "__0_penManager__param",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 114
    },
    "manager": {
      "name": "manager",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 12
    },
    "__intnl_SystemInt32_11": {
      "name": "__intnl_SystemInt32_11",
      "type": "System.Int32",
      "address": 755
    },
    "__intnl_SystemInt32_31": {
      "name": "__intnl_SystemInt32_31",
      "type": "System.Int32",
      "address": 918
    },
    "__intnl_SystemInt32_21": {
      "name": "__intnl_SystemInt32_21",
      "type": "System.Int32",
      "address": 828
    },
    "__intnl_SystemInt32_51": {
      "name": "__intnl_SystemInt32_51",
      "type": "System.Int32",
      "address": 969
    },
    "__intnl_SystemInt32_41": {
      "name": "__intnl_SystemInt32_41",
      "type": "System.Int32",
      "address": 950
    },
    "__intnl_SystemInt32_61": {
      "name": "__intnl_SystemInt32_61",
      "type": "System.Int32",
      "address": 996
    },
    "trailRenderer": {
      "name": "trailRenderer",
      "type": "UnityEngine.TrailRenderer",
      "address": 3
    },
    "__lcl_inkIdVector_UnityEngineVector3_3": {
      "name": "__lcl_inkIdVector_UnityEngineVector3_3",
      "type": "UnityEngine.Vector3",
      "address": 871
    },
    "__lcl_udonComponents_UnityEngineComponentArray_0": {
      "name": "__lcl_udonComponents_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 892
    },
    "__29__intnlparam": {
      "name": "__29__intnlparam",
      "type": "System.Int32",
      "address": 414
    },
    "__intnl_SystemString_8": {
      "name": "__intnl_SystemString_8",
      "type": "System.String",
      "address": 737
    },
    "__0_get_localPlayer__ret": {
      "name": "__0_get_localPlayer__ret",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 102
    },
    "__lcl_udon_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_udon_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 897
    },
    "__gintnl_SystemUInt32Array_1": {
      "name": "__gintnl_SystemUInt32Array_1",
      "type": "System.UInt32[]",
      "address": 502
    },
    "__this_VRCUdonUdonBehaviour_11": {
      "name": "__this_VRCUdonUdonBehaviour_11",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 252
    },
    "isPointerEnabled": {
      "name": "isPointerEnabled",
      "type": "System.Boolean",
      "address": 34
    },
    "__lcl_distance_SystemSingle_1": {
      "name": "__lcl_distance_SystemSingle_1",
      "type": "System.Single",
      "address": 681
    },
    "__lcl_distance_SystemSingle_0": {
      "name": "__lcl_distance_SystemSingle_0",
      "type": "System.Single",
      "address": 633
    },
    "__const_SystemString_36": {
      "name": "__const_SystemString_36",
      "type": "System.String",
      "address": 228
    },
    "__const_SystemString_37": {
      "name": "__const_SystemString_37",
      "type": "System.String",
      "address": 229
    },
    "__const_SystemString_34": {
      "name": "__const_SystemString_34",
      "type": "System.String",
      "address": 226
    },
    "__const_SystemString_35": {
      "name": "__const_SystemString_35",
      "type": "System.String",
      "address": 227
    },
    "__const_SystemString_32": {
      "name": "__const_SystemString_32",
      "type": "System.String",
      "address": 209
    },
    "__const_SystemString_33": {
      "name": "__const_SystemString_33",
      "type": "System.String",
      "address": 225
    },
    "__const_SystemString_30": {
      "name": "__const_SystemString_30",
      "type": "System.String",
      "address": 198
    },
    "__const_SystemString_31": {
      "name": "__const_SystemString_31",
      "type": "System.String",
      "address": 208
    },
    "__const_SystemString_38": {
      "name": "__const_SystemString_38",
      "type": "System.String",
      "address": 230
    },
    "__const_SystemString_39": {
      "name": "__const_SystemString_39",
      "type": "System.String",
      "address": 232
    },
    "__lcl_tmpWidthMultiplier_SystemSingle_0": {
      "name": "__lcl_tmpWidthMultiplier_SystemSingle_0",
      "type": "System.Single",
      "address": 852
    },
    "__intnl_UnityEngineVector3_10": {
      "name": "__intnl_UnityEngineVector3_10",
      "type": "UnityEngine.Vector3",
      "address": 627
    },
    "__intnl_UnityEngineVector3_11": {
      "name": "__intnl_UnityEngineVector3_11",
      "type": "UnityEngine.Vector3",
      "address": 628
    },
    "__intnl_UnityEngineVector3_12": {
      "name": "__intnl_UnityEngineVector3_12",
      "type": "UnityEngine.Vector3",
      "address": 629
    },
    "__intnl_UnityEngineVector3_13": {
      "name": "__intnl_UnityEngineVector3_13",
      "type": "UnityEngine.Vector3",
      "address": 630
    },
    "__intnl_UnityEngineVector3_14": {
      "name": "__intnl_UnityEngineVector3_14",
      "type": "UnityEngine.Vector3",
      "address": 643
    },
    "__intnl_UnityEngineVector3_15": {
      "name": "__intnl_UnityEngineVector3_15",
      "type": "UnityEngine.Vector3",
      "address": 682
    },
    "__intnl_UnityEngineVector3_16": {
      "name": "__intnl_UnityEngineVector3_16",
      "type": "UnityEngine.Vector3",
      "address": 683
    },
    "__intnl_UnityEngineVector3_17": {
      "name": "__intnl_UnityEngineVector3_17",
      "type": "UnityEngine.Vector3",
      "address": 684
    },
    "__intnl_UnityEngineVector3_18": {
      "name": "__intnl_UnityEngineVector3_18",
      "type": "UnityEngine.Vector3",
      "address": 687
    },
    "__intnl_UnityEngineVector3_19": {
      "name": "__intnl_UnityEngineVector3_19",
      "type": "UnityEngine.Vector3",
      "address": 699
    },
    "__intnl_SystemType_2": {
      "name": "__intnl_SystemType_2",
      "type": "System.Type",
      "address": 934
    },
    "__intnl_SystemSingle_9": {
      "name": "__intnl_SystemSingle_9",
      "type": "System.Single",
      "address": 602
    },
    "__gintnl_SystemUInt32_118": {
      "name": "__gintnl_SystemUInt32_118",
      "type": "System.UInt32",
      "address": 412
    },
    "__gintnl_SystemUInt32_119": {
      "name": "__gintnl_SystemUInt32_119",
      "type": "System.UInt32",
      "address": 413
    },
    "__gintnl_SystemUInt32_114": {
      "name": "__gintnl_SystemUInt32_114",
      "type": "System.UInt32",
      "address": 394
    },
    "__gintnl_SystemUInt32_115": {
      "name": "__gintnl_SystemUInt32_115",
      "type": "System.UInt32",
      "address": 398
    },
    "__gintnl_SystemUInt32_116": {
      "name": "__gintnl_SystemUInt32_116",
      "type": "System.UInt32",
      "address": 404
    },
    "__gintnl_SystemUInt32_117": {
      "name": "__gintnl_SystemUInt32_117",
      "type": "System.UInt32",
      "address": 411
    },
    "__gintnl_SystemUInt32_110": {
      "name": "__gintnl_SystemUInt32_110",
      "type": "System.UInt32",
      "address": 388
    },
    "__gintnl_SystemUInt32_111": {
      "name": "__gintnl_SystemUInt32_111",
      "type": "System.UInt32",
      "address": 389
    },
    "__gintnl_SystemUInt32_112": {
      "name": "__gintnl_SystemUInt32_112",
      "type": "System.UInt32",
      "address": 392
    },
    "__gintnl_SystemUInt32_113": {
      "name": "__gintnl_SystemUInt32_113",
      "type": "System.UInt32",
      "address": 393
    },
    "__const_SystemString_86": {
      "name": "__const_SystemString_86",
      "type": "System.String",
      "address": 503
    },
    "__const_SystemString_87": {
      "name": "__const_SystemString_87",
      "type": "System.String",
      "address": 504
    },
    "__const_SystemString_84": {
      "name": "__const_SystemString_84",
      "type": "System.String",
      "address": 494
    },
    "__const_SystemString_85": {
      "name": "__const_SystemString_85",
      "type": "System.String",
      "address": 501
    },
    "__const_SystemString_82": {
      "name": "__const_SystemString_82",
      "type": "System.String",
      "address": 492
    },
    "__const_SystemString_83": {
      "name": "__const_SystemString_83",
      "type": "System.String",
      "address": 493
    },
    "__const_SystemString_80": {
      "name": "__const_SystemString_80",
      "type": "System.String",
      "address": 485
    },
    "__const_SystemString_81": {
      "name": "__const_SystemString_81",
      "type": "System.String",
      "address": 491
    },
    "__const_SystemString_88": {
      "name": "__const_SystemString_88",
      "type": "System.String",
      "address": 505
    },
    "__const_SystemString_89": {
      "name": "__const_SystemString_89",
      "type": "System.String",
      "address": 512
    },
    "__0_get_pointerRadiusMultiplierForDesktop__ret": {
      "name": "__0_get_pointerRadiusMultiplierForDesktop__ret",
      "type": "System.Single",
      "address": 87
    },
    "__intnl_SystemInt64_0": {
      "name": "__intnl_SystemInt64_0",
      "type": "System.Int64",
      "address": 938
    },
    "__intnl_VRCUdonUdonBehaviour_4": {
      "name": "__intnl_VRCUdonUdonBehaviour_4",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 689
    },
    "__2_inkIdVector__param": {
      "name": "__2_inkIdVector__param",
      "type": "UnityEngine.Vector3",
      "address": 420
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "__intnl_UnityEngineGameObject_12": {
      "name": "__intnl_UnityEngineGameObject_12",
      "type": "UnityEngine.GameObject",
      "address": 740
    },
    "__intnl_UnityEngineTransform_7": {
      "name": "__intnl_UnityEngineTransform_7",
      "type": "UnityEngine.Transform",
      "address": 566
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 132
    },
    "__const_UnityEngineKeyCode_0": {
      "name": "__const_UnityEngineKeyCode_0",
      "type": "UnityEngine.KeyCode",
      "address": 174
    },
    "__26__intnlparam": {
      "name": "__26__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 401
    },
    "__gintnl_SystemUInt32_99": {
      "name": "__gintnl_SystemUInt32_99",
      "type": "System.UInt32",
      "address": 354
    },
    "__gintnl_SystemUInt32_89": {
      "name": "__gintnl_SystemUInt32_89",
      "type": "System.UInt32",
      "address": 338
    },
    "__gintnl_SystemUInt32_59": {
      "name": "__gintnl_SystemUInt32_59",
      "type": "System.UInt32",
      "address": 289
    },
    "__gintnl_SystemUInt32_49": {
      "name": "__gintnl_SystemUInt32_49",
      "type": "System.UInt32",
      "address": 271
    },
    "__gintnl_SystemUInt32_79": {
      "name": "__gintnl_SystemUInt32_79",
      "type": "System.UInt32",
      "address": 314
    },
    "__gintnl_SystemUInt32_69": {
      "name": "__gintnl_SystemUInt32_69",
      "type": "System.UInt32",
      "address": 299
    },
    "__gintnl_SystemUInt32_19": {
      "name": "__gintnl_SystemUInt32_19",
      "type": "System.UInt32",
      "address": 155
    },
    "__gintnl_SystemUInt32_39": {
      "name": "__gintnl_SystemUInt32_39",
      "type": "System.UInt32",
      "address": 240
    },
    "__gintnl_SystemUInt32_29": {
      "name": "__gintnl_SystemUInt32_29",
      "type": "System.UInt32",
      "address": 201
    },
    "__const_SystemSingle_4": {
      "name": "__const_SystemSingle_4",
      "type": "System.Single",
      "address": 166
    },
    "__intnl_UnityEngineRectTransform_0": {
      "name": "__intnl_UnityEngineRectTransform_0",
      "type": "UnityEngine.RectTransform",
      "address": 606
    },
    "__intnl_UnityEngineComponent_2": {
      "name": "__intnl_UnityEngineComponent_2",
      "type": "UnityEngine.Component",
      "address": 939
    },
    "__0_get_IsUser__ret": {
      "name": "__0_get_IsUser__ret",
      "type": "System.Boolean",
      "address": 93
    },
    "__intnl_UnityEngineTransform_30": {
      "name": "__intnl_UnityEngineTransform_30",
      "type": "UnityEngine.Transform",
      "address": 999
    },
    "__const_SystemInt32_18": {
      "name": "__const_SystemInt32_18",
      "type": "System.Int32",
      "address": 499
    },
    "__gintnl_SystemUInt32_92": {
      "name": "__gintnl_SystemUInt32_92",
      "type": "System.UInt32",
      "address": 341
    },
    "__gintnl_SystemUInt32_82": {
      "name": "__gintnl_SystemUInt32_82",
      "type": "System.UInt32",
      "address": 323
    },
    "__gintnl_SystemUInt32_52": {
      "name": "__gintnl_SystemUInt32_52",
      "type": "System.UInt32",
      "address": 281
    },
    "__gintnl_SystemUInt32_42": {
      "name": "__gintnl_SystemUInt32_42",
      "type": "System.UInt32",
      "address": 256
    },
    "__gintnl_SystemUInt32_72": {
      "name": "__gintnl_SystemUInt32_72",
      "type": "System.UInt32",
      "address": 303
    },
    "__gintnl_SystemUInt32_62": {
      "name": "__gintnl_SystemUInt32_62",
      "type": "System.UInt32",
      "address": 292
    },
    "__gintnl_SystemUInt32_12": {
      "name": "__gintnl_SystemUInt32_12",
      "type": "System.UInt32",
      "address": 128
    },
    "__gintnl_SystemUInt32_32": {
      "name": "__gintnl_SystemUInt32_32",
      "type": "System.UInt32",
      "address": 210
    },
    "__gintnl_SystemUInt32_22": {
      "name": "__gintnl_SystemUInt32_22",
      "type": "System.UInt32",
      "address": 173
    },
    "__const_UnityEngineVector3_2": {
      "name": "__const_UnityEngineVector3_2",
      "type": "UnityEngine.Vector3",
      "address": 206
    },
    "__intnl_SystemUInt32_0": {
      "name": "__intnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 709
    },
    "canBeErasedWithOtherPointers": {
      "name": "canBeErasedWithOtherPointers",
      "type": "System.Boolean",
      "address": 22
    },
    "__lcl_i_SystemInt32_1": {
      "name": "__lcl_i_SystemInt32_1",
      "type": "System.Int32",
      "address": 800
    },
    "__lcl_i_SystemInt32_0": {
      "name": "__lcl_i_SystemInt32_0",
      "type": "System.Int32",
      "address": 645
    },
    "__lcl_i_SystemInt32_2": {
      "name": "__lcl_i_SystemInt32_2",
      "type": "System.Int32",
      "address": 887
    },
    "__const_SystemInt32_15": {
      "name": "__const_SystemInt32_15",
      "type": "System.Int32",
      "address": 496
    },
    "__lcl_behaviour_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_behaviour_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 932
    },
    "_respawnEventName": {
      "name": "_respawnEventName",
      "type": "System.String",
      "address": 29
    },
    "__lcl_inkIdVector_UnityEngineVector3_0": {
      "name": "__lcl_inkIdVector_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 745
    },
    "__intnl_SystemString_5": {
      "name": "__intnl_SystemString_5",
      "type": "System.String",
      "address": 710
    },
    "headTracking": {
      "name": "headTracking",
      "type": "VRC.SDKBase.VRCPlayerApi+TrackingData",
      "address": 56
    },
    "__lcl_unique_SystemString_0": {
      "name": "__lcl_unique_SystemString_0",
      "type": "System.String",
      "address": 535
    },
    "__this_VRCUdonUdonBehaviour_16": {
      "name": "__this_VRCUdonUdonBehaviour_16",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 278
    },
    "__const_SystemType_5": {
      "name": "__const_SystemType_5",
      "type": "System.Type",
      "address": 192
    },
    "__intnl_SystemSingle_4": {
      "name": "__intnl_SystemSingle_4",
      "type": "System.Single",
      "address": 522
    },
    "__gintnl_SystemUInt32_128": {
      "name": "__gintnl_SystemUInt32_128",
      "type": "System.UInt32",
      "address": 431
    },
    "__gintnl_SystemUInt32_129": {
      "name": "__gintnl_SystemUInt32_129",
      "type": "System.UInt32",
      "address": 432
    },
    "__gintnl_SystemUInt32_124": {
      "name": "__gintnl_SystemUInt32_124",
      "type": "System.UInt32",
      "address": 425
    },
    "__gintnl_SystemUInt32_125": {
      "name": "__gintnl_SystemUInt32_125",
      "type": "System.UInt32",
      "address": 426
    },
    "__gintnl_SystemUInt32_126": {
      "name": "__gintnl_SystemUInt32_126",
      "type": "System.UInt32",
      "address": 427
    },
    "__gintnl_SystemUInt32_127": {
      "name": "__gintnl_SystemUInt32_127",
      "type": "System.UInt32",
      "address": 428
    },
    "__gintnl_SystemUInt32_120": {
      "name": "__gintnl_SystemUInt32_120",
      "type": "System.UInt32",
      "address": 416
    },
    "__gintnl_SystemUInt32_121": {
      "name": "__gintnl_SystemUInt32_121",
      "type": "System.UInt32",
      "address": 421
    },
    "__gintnl_SystemUInt32_122": {
      "name": "__gintnl_SystemUInt32_122",
      "type": "System.UInt32",
      "address": 422
    },
    "__gintnl_SystemUInt32_123": {
      "name": "__gintnl_SystemUInt32_123",
      "type": "System.UInt32",
      "address": 423
    },
    "__1__intnlparam": {
      "name": "__1__intnlparam",
      "type": "System.Int32",
      "address": 110
    },
    "__intnl_SystemObject_4": {
      "name": "__intnl_SystemObject_4",
      "type": "System.Object",
      "address": 569
    },
    "__9_data__param": {
      "name": "__9_data__param",
      "type": "UnityEngine.Vector3[]",
      "address": 372
    },
    "__33__intnlparam": {
      "name": "__33__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 437
    },
    "__intnl_VRCUdonUdonBehaviour_3": {
      "name": "__intnl_VRCUdonUdonBehaviour_3",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 586
    },
    "__lcl_mode_SystemInt32_1": {
      "name": "__lcl_mode_SystemInt32_1",
      "type": "System.Int32",
      "address": 795
    },
    "__intnl_SystemBoolean_109": {
      "name": "__intnl_SystemBoolean_109",
      "type": "System.Boolean",
      "address": 987
    },
    "__intnl_SystemBoolean_108": {
      "name": "__intnl_SystemBoolean_108",
      "type": "System.Boolean",
      "address": 986
    },
    "__intnl_SystemBoolean_101": {
      "name": "__intnl_SystemBoolean_101",
      "type": "System.Boolean",
      "address": 943
    },
    "__intnl_SystemBoolean_100": {
      "name": "__intnl_SystemBoolean_100",
      "type": "System.Boolean",
      "address": 941
    },
    "__intnl_SystemBoolean_103": {
      "name": "__intnl_SystemBoolean_103",
      "type": "System.Boolean",
      "address": 946
    },
    "__intnl_SystemBoolean_102": {
      "name": "__intnl_SystemBoolean_102",
      "type": "System.Boolean",
      "address": 944
    },
    "__intnl_SystemBoolean_105": {
      "name": "__intnl_SystemBoolean_105",
      "type": "System.Boolean",
      "address": 957
    },
    "__intnl_SystemBoolean_104": {
      "name": "__intnl_SystemBoolean_104",
      "type": "System.Boolean",
      "address": 951
    },
    "__intnl_SystemBoolean_107": {
      "name": "__intnl_SystemBoolean_107",
      "type": "System.Boolean",
      "address": 983
    },
    "__intnl_SystemBoolean_106": {
      "name": "__intnl_SystemBoolean_106",
      "type": "System.Boolean",
      "address": 980
    },
    "__intnl_UnityEngineGameObject_15": {
      "name": "__intnl_UnityEngineGameObject_15",
      "type": "UnityEngine.GameObject",
      "address": 788
    },
    "__intnl_UnityEngineTransform_4": {
      "name": "__intnl_UnityEngineTransform_4",
      "type": "UnityEngine.Transform",
      "address": 559
    },
    "__18__intnlparam": {
      "name": "__18__intnlparam",
      "type": "UnityEngine.Vector3[]",
      "address": 386
    },
    "__intnl_SystemDouble_7": {
      "name": "__intnl_SystemDouble_7",
      "type": "System.Double",
      "address": 823
    },
    "__const_SystemString_9": {
      "name": "__const_SystemString_9",
      "type": "System.String",
      "address": 141
    },
    "__intnl_UnityEngineTransform_9": {
      "name": "__intnl_UnityEngineTransform_9",
      "type": "UnityEngine.Transform",
      "address": 610
    },
    "__intnl_UnityEngineVector3_2": {
      "name": "__intnl_UnityEngineVector3_2",
      "type": "UnityEngine.Vector3",
      "address": 567
    },
    "__lcl_data_UnityEngineVector3Array_1": {
      "name": "__lcl_data_UnityEngineVector3Array_1",
      "type": "UnityEngine.Vector3[]",
      "address": 754
    },
    "__intnl_SystemBoolean_82": {
      "name": "__intnl_SystemBoolean_82",
      "type": "System.Boolean",
      "address": 845
    },
    "__intnl_SystemBoolean_92": {
      "name": "__intnl_SystemBoolean_92",
      "type": "System.Boolean",
      "address": 898
    },
    "__intnl_SystemBoolean_12": {
      "name": "__intnl_SystemBoolean_12",
      "type": "System.Boolean",
      "address": 593
    },
    "__intnl_SystemBoolean_22": {
      "name": "__intnl_SystemBoolean_22",
      "type": "System.Boolean",
      "address": 614
    },
    "__intnl_SystemBoolean_32": {
      "name": "__intnl_SystemBoolean_32",
      "type": "System.Boolean",
      "address": 670
    },
    "__intnl_SystemBoolean_42": {
      "name": "__intnl_SystemBoolean_42",
      "type": "System.Boolean",
      "address": 704
    },
    "__intnl_SystemBoolean_52": {
      "name": "__intnl_SystemBoolean_52",
      "type": "System.Boolean",
      "address": 723
    },
    "__intnl_SystemBoolean_62": {
      "name": "__intnl_SystemBoolean_62",
      "type": "System.Boolean",
      "address": 734
    },
    "__intnl_SystemBoolean_72": {
      "name": "__intnl_SystemBoolean_72",
      "type": "System.Boolean",
      "address": 792
    },
    "__0_ink__param": {
      "name": "__0_ink__param",
      "type": "UnityEngine.GameObject",
      "address": 218
    },
    "__lcl_inkPositionPosition_UnityEngineVector3_0": {
      "name": "__lcl_inkPositionPosition_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 631
    },
    "clickPosInterval": {
      "name": "clickPosInterval",
      "type": "System.Single",
      "address": 38
    },
    "surftraceTarget": {
      "name": "surftraceTarget",
      "type": "UnityEngine.Collider",
      "address": 71
    },
    "__gintnl_SystemUInt32_97": {
      "name": "__gintnl_SystemUInt32_97",
      "type": "System.UInt32",
      "address": 352
    },
    "__gintnl_SystemUInt32_87": {
      "name": "__gintnl_SystemUInt32_87",
      "type": "System.UInt32",
      "address": 336
    },
    "__gintnl_SystemUInt32_57": {
      "name": "__gintnl_SystemUInt32_57",
      "type": "System.UInt32",
      "address": 287
    },
    "__gintnl_SystemUInt32_47": {
      "name": "__gintnl_SystemUInt32_47",
      "type": "System.UInt32",
      "address": 268
    },
    "__gintnl_SystemUInt32_77": {
      "name": "__gintnl_SystemUInt32_77",
      "type": "System.UInt32",
      "address": 310
    },
    "__gintnl_SystemUInt32_67": {
      "name": "__gintnl_SystemUInt32_67",
      "type": "System.UInt32",
      "address": 297
    },
    "__gintnl_SystemUInt32_17": {
      "name": "__gintnl_SystemUInt32_17",
      "type": "System.UInt32",
      "address": 150
    },
    "__gintnl_SystemUInt32_37": {
      "name": "__gintnl_SystemUInt32_37",
      "type": "System.UInt32",
      "address": 236
    },
    "__gintnl_SystemUInt32_27": {
      "name": "__gintnl_SystemUInt32_27",
      "type": "System.UInt32",
      "address": 190
    },
    "__const_UnityEngineVector3_1": {
      "name": "__const_UnityEngineVector3_1",
      "type": "UnityEngine.Vector3",
      "address": 199
    },
    "__intnl_SystemInt32_16": {
      "name": "__intnl_SystemInt32_16",
      "type": "System.Int32",
      "address": 797
    },
    "__intnl_SystemInt32_36": {
      "name": "__intnl_SystemInt32_36",
      "type": "System.Int32",
      "address": 929
    },
    "__intnl_SystemInt32_26": {
      "name": "__intnl_SystemInt32_26",
      "type": "System.Int32",
      "address": 886
    },
    "__intnl_SystemInt32_56": {
      "name": "__intnl_SystemInt32_56",
      "type": "System.Int32",
      "address": 979
    },
    "__intnl_SystemInt32_46": {
      "name": "__intnl_SystemInt32_46",
      "type": "System.Int32",
      "address": 960
    },
    "__11__intnlparam": {
      "name": "__11__intnlparam",
      "type": "System.Int32",
      "address": 330
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "_pointerRadiusMultiplierForDesktop": {
      "name": "_pointerRadiusMultiplierForDesktop",
      "type": "System.Single",
      "address": 17
    },
    "__gintnl_SystemObjectArray_0": {
      "name": "__gintnl_SystemObjectArray_0",
      "type": "System.Object[]",
      "address": 489
    },
    "penId": {
      "name": "penId",
      "type": "System.Int32",
      "address": 42
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 542
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 538
    },
    "__intnl_SystemInt32_3": {
      "name": "__intnl_SystemInt32_3",
      "type": "System.Int32",
      "address": 554
    },
    "__intnl_SystemInt32_2": {
      "name": "__intnl_SystemInt32_2",
      "type": "System.Int32",
      "address": 546
    },
    "__intnl_SystemInt32_5": {
      "name": "__intnl_SystemInt32_5",
      "type": "System.Int32",
      "address": 673
    },
    "__intnl_SystemInt32_4": {
      "name": "__intnl_SystemInt32_4",
      "type": "System.Int32",
      "address": 666
    },
    "__intnl_SystemInt32_7": {
      "name": "__intnl_SystemInt32_7",
      "type": "System.Int32",
      "address": 675
    },
    "__intnl_SystemInt32_6": {
      "name": "__intnl_SystemInt32_6",
      "type": "System.Int32",
      "address": 674
    },
    "__intnl_SystemInt32_9": {
      "name": "__intnl_SystemInt32_9",
      "type": "System.Int32",
      "address": 743
    },
    "__intnl_SystemInt32_8": {
      "name": "__intnl_SystemInt32_8",
      "type": "System.Int32",
      "address": 738
    },
    "__intnl_SystemString_2": {
      "name": "__intnl_SystemString_2",
      "type": "System.String",
      "address": 589
    },
    "__lcl_z_SystemInt32_0": {
      "name": "__lcl_z_SystemInt32_0",
      "type": "System.Int32",
      "address": 909
    },
    "__const_SystemType_6": {
      "name": "__const_SystemType_6",
      "type": "System.Type",
      "address": 216
    },
    "__const_SystemString_56": {
      "name": "__const_SystemString_56",
      "type": "System.String",
      "address": 325
    },
    "__const_SystemString_57": {
      "name": "__const_SystemString_57",
      "type": "System.String",
      "address": 357
    },
    "__const_SystemString_54": {
      "name": "__const_SystemString_54",
      "type": "System.String",
      "address": 312
    },
    "__const_SystemString_55": {
      "name": "__const_SystemString_55",
      "type": "System.String",
      "address": 313
    },
    "__const_SystemString_52": {
      "name": "__const_SystemString_52",
      "type": "System.String",
      "address": 306
    },
    "__const_SystemString_53": {
      "name": "__const_SystemString_53",
      "type": "System.String",
      "address": 307
    },
    "__const_SystemString_50": {
      "name": "__const_SystemString_50",
      "type": "System.String",
      "address": 269
    },
    "__const_SystemString_51": {
      "name": "__const_SystemString_51",
      "type": "System.String",
      "address": 273
    },
    "__const_SystemString_58": {
      "name": "__const_SystemString_58",
      "type": "System.String",
      "address": 358
    },
    "__const_SystemString_59": {
      "name": "__const_SystemString_59",
      "type": "System.String",
      "address": 380
    },
    "__1_o__param": {
      "name": "__1_o__param",
      "type": "System.Object",
      "address": 405
    },
    "__0_o__param": {
      "name": "__0_o__param",
      "type": "System.Object",
      "address": 187
    },
    "__0_c__param": {
      "name": "__0_c__param",
      "type": "UnityEngine.Color",
      "address": 484
    },
    "__intnl_SystemSingle_3": {
      "name": "__intnl_SystemSingle_3",
      "type": "System.Single",
      "address": 521
    },
    "__2_o__param": {
      "name": "__2_o__param",
      "type": "System.Object",
      "address": 246
    },
    "inkPoolSynced": {
      "name": "inkPoolSynced",
      "type": "UnityEngine.Transform",
      "address": 9
    },
    "__intnl_UnityEngineMeshCollider_0": {
      "name": "__intnl_UnityEngineMeshCollider_0",
      "type": "UnityEngine.MeshCollider",
      "address": 679
    },
    "__intnl_UnityEngineMeshCollider_1": {
      "name": "__intnl_UnityEngineMeshCollider_1",
      "type": "UnityEngine.MeshCollider",
      "address": 776
    },
    "__intnl_UnityEngineMeshCollider_2": {
      "name": "__intnl_UnityEngineMeshCollider_2",
      "type": "UnityEngine.MeshCollider",
      "address": 825
    },
    "__intnl_UnityEngineMeshCollider_3": {
      "name": "__intnl_UnityEngineMeshCollider_3",
      "type": "UnityEngine.MeshCollider",
      "address": 850
    },
    "__1_ownerIdVector__param": {
      "name": "__1_ownerIdVector__param",
      "type": "UnityEngine.Vector3",
      "address": 347
    },
    "__0_get_objectSync__ret": {
      "name": "__0_get_objectSync__ret",
      "type": "VRC.SDK3.Components.VRCObjectSync",
      "address": 97
    },
    "__intnl_SystemObject_3": {
      "name": "__intnl_SystemObject_3",
      "type": "System.Object",
      "address": 568
    },
    "__this_UnityEngineTransform_0": {
      "name": "__this_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 207
    },
    "isScreenMode": {
      "name": "isScreenMode",
      "type": "System.Boolean",
      "address": 67
    },
    "_pickup": {
      "name": "_pickup",
      "type": "VRC.SDK3.Components.VRCPickup",
      "address": 26
    },
    "__7_data__param": {
      "name": "__7_data__param",
      "type": "UnityEngine.Vector3[]",
      "address": 366
    },
    "__lcl_targetID_SystemInt64_0": {
      "name": "__lcl_targetID_SystemInt64_0",
      "type": "System.Int64",
      "address": 928
    },
    "__lcl_penIdVector_UnityEngineVector3_2": {
      "name": "__lcl_penIdVector_UnityEngineVector3_2",
      "type": "UnityEngine.Vector3",
      "address": 870
    },
    "__lcl_inkMeshLayer_SystemInt32_0": {
      "name": "__lcl_inkMeshLayer_SystemInt32_0",
      "type": "System.Int32",
      "address": 772
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 117
    },
    "__const_UnityEngineKeyCode_5": {
      "name": "__const_UnityEngineKeyCode_5",
      "type": "UnityEngine.KeyCode",
      "address": 189
    },
    "__intnl_UnityEngineGameObject_18": {
      "name": "__intnl_UnityEngineGameObject_18",
      "type": "UnityEngine.GameObject",
      "address": 826
    },
    "__intnl_UnityEngineTransform_1": {
      "name": "__intnl_UnityEngineTransform_1",
      "type": "UnityEngine.Transform",
      "address": 525
    },
    "__0_get_pointerRadius__ret": {
      "name": "__0_get_pointerRadius__ret",
      "type": "System.Single",
      "address": 78
    },
    "__30__intnlparam": {
      "name": "__30__intnlparam",
      "type": "UnityEngine.Vector3[]",
      "address": 415
    },
    "__lcl_playerIdVector_UnityEngineVector3_0": {
      "name": "__lcl_playerIdVector_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 815
    },
    "__25__intnlparam": {
      "name": "__25__intnlparam",
      "type": "UnityEngine.GameObject",
      "address": 400
    },
    "__const_SystemSingle_3": {
      "name": "__const_SystemSingle_3",
      "type": "System.Single",
      "address": 157
    },
    "__0_get_localPlayerIdVector__ret": {
      "name": "__0_get_localPlayerIdVector__ret",
      "type": "UnityEngine.Vector3",
      "address": 107
    },
    "__intnl_SystemDouble_0": {
      "name": "__intnl_SystemDouble_0",
      "type": "System.Double",
      "address": 539
    },
    "__lcl_idValue_SystemObject_0": {
      "name": "__lcl_idValue_SystemObject_0",
      "type": "System.Object",
      "address": 936
    },
    "__0__TakeOwnership__ret": {
      "name": "__0__TakeOwnership__ret",
      "type": "System.Boolean",
      "address": 302
    },
    "__5__intnlparam": {
      "name": "__5__intnlparam",
      "type": "System.Int32",
      "address": 126
    },
    "__intnl_UnityEngineVector3_1": {
      "name": "__intnl_UnityEngineVector3_1",
      "type": "UnityEngine.Vector3",
      "address": 565
    },
    "__intnl_SystemString_12": {
      "name": "__intnl_SystemString_12",
      "type": "System.String",
      "address": 901
    },
    "__intnl_SystemString_13": {
      "name": "__intnl_SystemString_13",
      "type": "System.String",
      "address": 903
    },
    "__intnl_SystemString_10": {
      "name": "__intnl_SystemString_10",
      "type": "System.String",
      "address": 843
    },
    "__intnl_SystemString_11": {
      "name": "__intnl_SystemString_11",
      "type": "System.String",
      "address": 899
    },
    "pointerMaterialNormal": {
      "name": "pointerMaterialNormal",
      "type": "UnityEngine.Material",
      "address": 18
    },
    "__lcl_data_UnityEngineVector3Array_4": {
      "name": "__lcl_data_UnityEngineVector3Array_4",
      "type": "UnityEngine.Vector3[]",
      "address": 862
    },
    "__intnl_UnityEngineVector2_0": {
      "name": "__intnl_UnityEngineVector2_0",
      "type": "UnityEngine.Vector2",
      "address": 624
    },
    "__intnl_SystemBoolean_85": {
      "name": "__intnl_SystemBoolean_85",
      "type": "System.Boolean",
      "address": 874
    },
    "__intnl_SystemBoolean_95": {
      "name": "__intnl_SystemBoolean_95",
      "type": "System.Boolean",
      "address": 917
    },
    "__intnl_SystemBoolean_15": {
      "name": "__intnl_SystemBoolean_15",
      "type": "System.Boolean",
      "address": 596
    },
    "__intnl_SystemBoolean_25": {
      "name": "__intnl_SystemBoolean_25",
      "type": "System.Boolean",
      "address": 651
    },
    "__intnl_SystemBoolean_35": {
      "name": "__intnl_SystemBoolean_35",
      "type": "System.Boolean",
      "address": 677
    },
    "__intnl_SystemBoolean_45": {
      "name": "__intnl_SystemBoolean_45",
      "type": "System.Boolean",
      "address": 708
    },
    "__intnl_SystemBoolean_55": {
      "name": "__intnl_SystemBoolean_55",
      "type": "System.Boolean",
      "address": 726
    },
    "__intnl_SystemBoolean_65": {
      "name": "__intnl_SystemBoolean_65",
      "type": "System.Boolean",
      "address": 749
    },
    "__intnl_SystemBoolean_75": {
      "name": "__intnl_SystemBoolean_75",
      "type": "System.Boolean",
      "address": 796
    },
    "__const_SystemInt32_12": {
      "name": "__const_SystemInt32_12",
      "type": "System.Int32",
      "address": 376
    },
    "__2_value__param": {
      "name": "__2_value__param",
      "type": "System.Boolean",
      "address": 276
    },
    "__intnl_SystemInt32_13": {
      "name": "__intnl_SystemInt32_13",
      "type": "System.Int32",
      "address": 771
    },
    "__intnl_SystemInt32_33": {
      "name": "__intnl_SystemInt32_33",
      "type": "System.Int32",
      "address": 921
    },
    "__intnl_SystemInt32_23": {
      "name": "__intnl_SystemInt32_23",
      "type": "System.Int32",
      "address": 839
    },
    "__intnl_SystemInt32_53": {
      "name": "__intnl_SystemInt32_53",
      "type": "System.Int32",
      "address": 973
    },
    "__intnl_SystemInt32_43": {
      "name": "__intnl_SystemInt32_43",
      "type": "System.Int32",
      "address": 954
    },
    "__intnl_SystemInt32_63": {
      "name": "__intnl_SystemInt32_63",
      "type": "System.Int32",
      "address": 1004
    },
    "__0___0__PackData__ret": {
      "name": "__0___0__PackData__ret",
      "type": "UnityEngine.Vector3[]",
      "address": 343
    },
    "__0_get_AllowCallPen__ret": {
      "name": "__0_get_AllowCallPen__ret",
      "type": "System.Boolean",
      "address": 76
    },
    "__intnl_SystemInt32_18": {
      "name": "__intnl_SystemInt32_18",
      "type": "System.Int32",
      "address": 805
    },
    "__intnl_SystemInt32_38": {
      "name": "__intnl_SystemInt32_38",
      "type": "System.Int32",
      "address": 947
    },
    "__intnl_SystemInt32_28": {
      "name": "__intnl_SystemInt32_28",
      "type": "System.Int32",
      "address": 910
    },
    "__intnl_SystemInt32_58": {
      "name": "__intnl_SystemInt32_58",
      "type": "System.Int32",
      "address": 988
    },
    "__intnl_SystemInt32_48": {
      "name": "__intnl_SystemInt32_48",
      "type": "System.Int32",
      "address": 963
    },
    "__17__intnlparam": {
      "name": "__17__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 385
    },
    "_alternativeObjectSync": {
      "name": "_alternativeObjectSync",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 28
    },
    "__this_VRCUdonUdonBehaviour_13": {
      "name": "__this_VRCUdonUdonBehaviour_13",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 262
    },
    "__39__intnlparam": {
      "name": "__39__intnlparam",
      "type": "UnityEngine.GameObject",
      "address": 514
    },
    "_wh": {
      "name": "_wh",
      "type": "UnityEngine.Vector2",
      "address": 60
    },
    "inkPoolRoot": {
      "name": "inkPoolRoot",
      "type": "UnityEngine.Transform",
      "address": 7
    },
    "__intnl_SystemType_0": {
      "name": "__intnl_SystemType_0",
      "type": "System.Type",
      "address": 678
    },
    "scalar": {
      "name": "scalar",
      "type": "System.Single",
      "address": 65
    },
    "__this_VRCUdonUdonBehaviour_18": {
      "name": "__this_VRCUdonUdonBehaviour_18",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 480
    },
    "__intnl_UnityEngineObject_10": {
      "name": "__intnl_UnityEngineObject_10",
      "type": "UnityEngine.Object",
      "address": 904
    },
    "__const_SystemType_3": {
      "name": "__const_SystemType_3",
      "type": "System.Type",
      "address": 99
    },
    "__22__intnlparam": {
      "name": "__22__intnlparam",
      "type": "UnityEngine.GameObject",
      "address": 395
    },
    "__intnl_VRCUdonUdonBehaviour_6": {
      "name": "__intnl_VRCUdonUdonBehaviour_6",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 693
    },
    "__intnl_UnityEngineLayerMask_0": {
      "name": "__intnl_UnityEngineLayerMask_0",
      "type": "UnityEngine.LayerMask",
      "address": 582
    },
    "__intnl_UnityEngineGameObject_10": {
      "name": "__intnl_UnityEngineGameObject_10",
      "type": "UnityEngine.GameObject",
      "address": 718
    },
    "inkColliderLayerMask": {
      "name": "inkColliderLayerMask",
      "type": "System.Int32",
      "address": 33
    },
    "__intnl_SystemDouble_8": {
      "name": "__intnl_SystemDouble_8",
      "type": "System.Double",
      "address": 829
    },
    "__lcl_x_SystemInt32_0": {
      "name": "__lcl_x_SystemInt32_0",
      "type": "System.Int32",
      "address": 907
    },
    "__const_SystemString_6": {
      "name": "__const_SystemString_6",
      "type": "System.String",
      "address": 134
    },
    "__const_UnityEngineKeyCode_2": {
      "name": "__const_UnityEngineKeyCode_2",
      "type": "UnityEngine.KeyCode",
      "address": 179
    },
    "__intnl_UnityEngineVector3_9": {
      "name": "__intnl_UnityEngineVector3_9",
      "type": "UnityEngine.Vector3",
      "address": 620
    },
    "__const_SystemSingle_6": {
      "name": "__const_SystemSingle_6",
      "type": "System.Single",
      "address": 193
    },
    "logColor": {
      "name": "logColor",
      "type": "UnityEngine.Color",
      "address": 74
    },
    "__36__intnlparam": {
      "name": "__36__intnlparam",
      "type": "System.String",
      "address": 487
    },
    "__intnl_UnityEngineVector3_4": {
      "name": "__intnl_UnityEngineVector3_4",
      "type": "UnityEngine.Vector3",
      "address": 611
    },
    "propertyBlock": {
      "name": "propertyBlock",
      "type": "UnityEngine.MaterialPropertyBlock",
      "address": 47
    },
    "__gintnl_SystemUInt32_90": {
      "name": "__gintnl_SystemUInt32_90",
      "type": "System.UInt32",
      "address": 339
    },
    "__gintnl_SystemUInt32_80": {
      "name": "__gintnl_SystemUInt32_80",
      "type": "System.UInt32",
      "address": 315
    },
    "__gintnl_SystemUInt32_50": {
      "name": "__gintnl_SystemUInt32_50",
      "type": "System.UInt32",
      "address": 272
    },
    "__gintnl_SystemUInt32_40": {
      "name": "__gintnl_SystemUInt32_40",
      "type": "System.UInt32",
      "address": 245
    },
    "__gintnl_SystemUInt32_70": {
      "name": "__gintnl_SystemUInt32_70",
      "type": "System.UInt32",
      "address": 300
    },
    "__gintnl_SystemUInt32_60": {
      "name": "__gintnl_SystemUInt32_60",
      "type": "System.UInt32",
      "address": 290
    },
    "__gintnl_SystemUInt32_10": {
      "name": "__gintnl_SystemUInt32_10",
      "type": "System.UInt32",
      "address": 123
    },
    "__gintnl_SystemUInt32_30": {
      "name": "__gintnl_SystemUInt32_30",
      "type": "System.UInt32",
      "address": 203
    },
    "__gintnl_SystemUInt32_20": {
      "name": "__gintnl_SystemUInt32_20",
      "type": "System.UInt32",
      "address": 161
    },
    "__intnl_SystemUInt32_2": {
      "name": "__intnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 945
    },
    "__lcl_other_UnityEngineCollider_1": {
      "name": "__lcl_other_UnityEngineCollider_1",
      "type": "UnityEngine.Collider",
      "address": 889
    },
    "__lcl_other_UnityEngineCollider_0": {
      "name": "__lcl_other_UnityEngineCollider_0",
      "type": "UnityEngine.Collider",
      "address": 647
    },
    "screenOverlay": {
      "name": "screenOverlay",
      "type": "UnityEngine.Canvas",
      "address": 20
    },
    "__intnl_VRCSDK3DataDataToken_0": {
      "name": "__intnl_VRCSDK3DataDataToken_0",
      "type": "VRC.SDK3.Data.DataToken",
      "address": 799
    },
    "__const_SystemInt32_17": {
      "name": "__const_SystemInt32_17",
      "type": "System.Int32",
      "address": 498
    },
    "__lcl_inkColliderLayer_SystemInt32_0": {
      "name": "__lcl_inkColliderLayer_SystemInt32_0",
      "type": "System.Int32",
      "address": 774
    },
    "__intnl_SystemInt32_10": {
      "name": "__intnl_SystemInt32_10",
      "type": "System.Int32",
      "address": 753
    },
    "__intnl_SystemInt32_30": {
      "name": "__intnl_SystemInt32_30",
      "type": "System.Int32",
      "address": 914
    },
    "__intnl_SystemInt32_20": {
      "name": "__intnl_SystemInt32_20",
      "type": "System.Int32",
      "address": 821
    },
    "__intnl_SystemInt32_50": {
      "name": "__intnl_SystemInt32_50",
      "type": "System.Int32",
      "address": 967
    },
    "__intnl_SystemInt32_40": {
      "name": "__intnl_SystemInt32_40",
      "type": "System.Int32",
      "address": 949
    },
    "__intnl_SystemInt32_60": {
      "name": "__intnl_SystemInt32_60",
      "type": "System.Int32",
      "address": 993
    },
    "__lcl_lineInstance_UnityEngineGameObject_0": {
      "name": "__lcl_lineInstance_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 816
    },
    "__lcl_inkIdVector_UnityEngineVector3_2": {
      "name": "__lcl_inkIdVector_UnityEngineVector3_2",
      "type": "UnityEngine.Vector3",
      "address": 860
    },
    "__intnl_SystemString_7": {
      "name": "__intnl_SystemString_7",
      "type": "System.String",
      "address": 712
    },
    "__gintnl_SystemUInt32Array_2": {
      "name": "__gintnl_SystemUInt32Array_2",
      "type": "System.UInt32[]",
      "address": 506
    },
    "__this_UnityEngineGameObject_0": {
      "name": "__this_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 122
    },
    "__this_VRCUdonUdonBehaviour_10": {
      "name": "__this_VRCUdonUdonBehaviour_10",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 244
    },
    "__1_value__param": {
      "name": "__1_value__param",
      "type": "System.Boolean",
      "address": 274
    },
    "__14__intnlparam": {
      "name": "__14__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 334
    },
    "__const_SystemString_26": {
      "name": "__const_SystemString_26",
      "type": "System.String",
      "address": 169
    },
    "__const_SystemString_27": {
      "name": "__const_SystemString_27",
      "type": "System.String",
      "address": 181
    },
    "__const_SystemString_24": {
      "name": "__const_SystemString_24",
      "type": "System.String",
      "address": 167
    },
    "__const_SystemString_25": {
      "name": "__const_SystemString_25",
      "type": "System.String",
      "address": 168
    },
    "__const_SystemString_22": {
      "name": "__const_SystemString_22",
      "type": "System.String",
      "address": 164
    },
    "__const_SystemString_23": {
      "name": "__const_SystemString_23",
      "type": "System.String",
      "address": 165
    },
    "__const_SystemString_20": {
      "name": "__const_SystemString_20",
      "type": "System.String",
      "address": 162
    },
    "__const_SystemString_21": {
      "name": "__const_SystemString_21",
      "type": "System.String",
      "address": 163
    },
    "__const_SystemString_28": {
      "name": "__const_SystemString_28",
      "type": "System.String",
      "address": 183
    },
    "__const_SystemString_29": {
      "name": "__const_SystemString_29",
      "type": "System.String",
      "address": 188
    },
    "__intnl_SystemSingle_6": {
      "name": "__intnl_SystemSingle_6",
      "type": "System.Single",
      "address": 537
    },
    "__gintnl_SystemUInt32_108": {
      "name": "__gintnl_SystemUInt32_108",
      "type": "System.UInt32",
      "address": 375
    },
    "__gintnl_SystemUInt32_109": {
      "name": "__gintnl_SystemUInt32_109",
      "type": "System.UInt32",
      "address": 384
    },
    "__gintnl_SystemUInt32_104": {
      "name": "__gintnl_SystemUInt32_104",
      "type": "System.UInt32",
      "address": 365
    },
    "__gintnl_SystemUInt32_105": {
      "name": "__gintnl_SystemUInt32_105",
      "type": "System.UInt32",
      "address": 368
    },
    "__gintnl_SystemUInt32_106": {
      "name": "__gintnl_SystemUInt32_106",
      "type": "System.UInt32",
      "address": 371
    },
    "__gintnl_SystemUInt32_107": {
      "name": "__gintnl_SystemUInt32_107",
      "type": "System.UInt32",
      "address": 374
    },
    "__gintnl_SystemUInt32_100": {
      "name": "__gintnl_SystemUInt32_100",
      "type": "System.UInt32",
      "address": 355
    },
    "__gintnl_SystemUInt32_101": {
      "name": "__gintnl_SystemUInt32_101",
      "type": "System.UInt32",
      "address": 356
    },
    "__gintnl_SystemUInt32_102": {
      "name": "__gintnl_SystemUInt32_102",
      "type": "System.UInt32",
      "address": 359
    },
    "__gintnl_SystemUInt32_103": {
      "name": "__gintnl_SystemUInt32_103",
      "type": "System.UInt32",
      "address": 362
    },
    "isRoundedTrailShader": {
      "name": "isRoundedTrailShader",
      "type": "System.Boolean",
      "address": 46
    },
    "penManager": {
      "name": "penManager",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 30
    },
    "_isUserInVR": {
      "name": "_isUserInVR",
      "type": "System.Boolean",
      "address": 54
    },
    "__intnl_SystemObject_6": {
      "name": "__intnl_SystemObject_6",
      "type": "System.Object",
      "address": 575
    },
    "__const_UnityEngineQuaternion_0": {
      "name": "__const_UnityEngineQuaternion_0",
      "type": "UnityEngine.Quaternion",
      "address": 200
    },
    "__0_get_isSurftraceMode__ret": {
      "name": "__0_get_isSurftraceMode__ret",
      "type": "System.Boolean",
      "address": 196
    },
    "__8_data__param": {
      "name": "__8_data__param",
      "type": "UnityEngine.Vector3[]",
      "address": 369
    },
    "__0_get_penIdVector__ret": {
      "name": "__0_get_penIdVector__ret",
      "type": "UnityEngine.Vector3",
      "address": 100
    },
    "__intnl_VRCUdonUdonBehaviour_5": {
      "name": "__intnl_VRCUdonUdonBehaviour_5",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 690
    },
    "__intnl_UnityEngineGameObject_13": {
      "name": "__intnl_UnityEngineGameObject_13",
      "type": "UnityEngine.GameObject",
      "address": 773
    },
    "__intnl_UnityEngineTransform_6": {
      "name": "__intnl_UnityEngineTransform_6",
      "type": "UnityEngine.Transform",
      "address": 563
    },
    "clampWH": {
      "name": "clampWH",
      "type": "UnityEngine.Vector2",
      "address": 62
    },
    "__intnl_VRCUdonUdonBehaviour_8": {
      "name": "__intnl_VRCUdonUdonBehaviour_8",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 742
    },
    "__intnl_SystemDouble_5": {
      "name": "__intnl_SystemDouble_5",
      "type": "System.Double",
      "address": 548
    },
    "_localPlayer": {
      "name": "_localPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 48
    },
    "currentSyncState": {
      "name": "currentSyncState",
      "type": "System.Int32",
      "address": 41
    },
    "__gintnl_SystemUInt32_98": {
      "name": "__gintnl_SystemUInt32_98",
      "type": "System.UInt32",
      "address": 353
    },
    "__gintnl_SystemUInt32_88": {
      "name": "__gintnl_SystemUInt32_88",
      "type": "System.UInt32",
      "address": 337
    },
    "__gintnl_SystemUInt32_58": {
      "name": "__gintnl_SystemUInt32_58",
      "type": "System.UInt32",
      "address": 288
    },
    "__gintnl_SystemUInt32_48": {
      "name": "__gintnl_SystemUInt32_48",
      "type": "System.UInt32",
      "address": 270
    },
    "__gintnl_SystemUInt32_78": {
      "name": "__gintnl_SystemUInt32_78",
      "type": "System.UInt32",
      "address": 311
    },
    "__gintnl_SystemUInt32_68": {
      "name": "__gintnl_SystemUInt32_68",
      "type": "System.UInt32",
      "address": 298
    },
    "__gintnl_SystemUInt32_18": {
      "name": "__gintnl_SystemUInt32_18",
      "type": "System.UInt32",
      "address": 153
    },
    "__gintnl_SystemUInt32_38": {
      "name": "__gintnl_SystemUInt32_38",
      "type": "System.UInt32",
      "address": 237
    },
    "__gintnl_SystemUInt32_28": {
      "name": "__gintnl_SystemUInt32_28",
      "type": "System.UInt32",
      "address": 195
    },
    "__1_inkIdVector__param": {
      "name": "__1_inkIdVector__param",
      "type": "UnityEngine.Vector3",
      "address": 346
    },
    "_isCheckedIsUserInVR": {
      "name": "_isCheckedIsUserInVR",
      "type": "System.Boolean",
      "address": 53
    },
    "__const_SystemSingle_5": {
      "name": "__const_SystemSingle_5",
      "type": "System.Single",
      "address": 185
    },
    "__2__intnlparam": {
      "name": "__2__intnlparam",
      "type": "UnityEngine.Transform",
      "address": 119
    },
    "__intnl_UnityEngineComponent_1": {
      "name": "__intnl_UnityEngineComponent_1",
      "type": "UnityEngine.Component",
      "address": 529
    },
    "__lcl_inkPoolRootGO_UnityEngineGameObject_0": {
      "name": "__lcl_inkPoolRootGO_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 532
    },
    "__intnl_SystemBoolean_80": {
      "name": "__intnl_SystemBoolean_80",
      "type": "System.Boolean",
      "address": 813
    },
    "__intnl_SystemBoolean_90": {
      "name": "__intnl_SystemBoolean_90",
      "type": "System.Boolean",
      "address": 894
    },
    "__intnl_SystemBoolean_10": {
      "name": "__intnl_SystemBoolean_10",
      "type": "System.Boolean",
      "address": 590
    },
    "__intnl_SystemBoolean_20": {
      "name": "__intnl_SystemBoolean_20",
      "type": "System.Boolean",
      "address": 601
    },
    "__intnl_SystemBoolean_30": {
      "name": "__intnl_SystemBoolean_30",
      "type": "System.Boolean",
      "address": 668
    },
    "__intnl_SystemBoolean_40": {
      "name": "__intnl_SystemBoolean_40",
      "type": "System.Boolean",
      "address": 696
    },
    "__intnl_SystemBoolean_50": {
      "name": "__intnl_SystemBoolean_50",
      "type": "System.Boolean",
      "address": 721
    },
    "__intnl_SystemBoolean_60": {
      "name": "__intnl_SystemBoolean_60",
      "type": "System.Boolean",
      "address": 731
    },
    "__intnl_SystemBoolean_70": {
      "name": "__intnl_SystemBoolean_70",
      "type": "System.Boolean",
      "address": 787
    },
    "__const_UnityEngineQueryTriggerInteraction_1": {
      "name": "__const_UnityEngineQueryTriggerInteraction_1",
      "type": "UnityEngine.QueryTriggerInteraction",
      "address": 472
    },
    "__intnl_UnityEngineQuaternion_0": {
      "name": "__intnl_UnityEngineQuaternion_0",
      "type": "UnityEngine.Quaternion",
      "address": 612
    },
    "__intnl_UnityEngineQuaternion_1": {
      "name": "__intnl_UnityEngineQuaternion_1",
      "type": "UnityEngine.Quaternion",
      "address": 639
    },
    "__intnl_UnityEngineQuaternion_2": {
      "name": "__intnl_UnityEngineQuaternion_2",
      "type": "UnityEngine.Quaternion",
      "address": 640
    },
    "__intnl_UnityEngineQuaternion_3": {
      "name": "__intnl_UnityEngineQuaternion_3",
      "type": "UnityEngine.Quaternion",
      "address": 641
    },
    "__intnl_UnityEngineQuaternion_4": {
      "name": "__intnl_UnityEngineQuaternion_4",
      "type": "UnityEngine.Quaternion",
      "address": 688
    },
    "__intnl_UnityEngineTransform_29": {
      "name": "__intnl_UnityEngineTransform_29",
      "type": "UnityEngine.Transform",
      "address": 995
    },
    "__intnl_UnityEngineTransform_28": {
      "name": "__intnl_UnityEngineTransform_28",
      "type": "UnityEngine.Transform",
      "address": 985
    },
    "__intnl_UnityEngineTransform_23": {
      "name": "__intnl_UnityEngineTransform_23",
      "type": "UnityEngine.Transform",
      "address": 842
    },
    "__intnl_UnityEngineTransform_22": {
      "name": "__intnl_UnityEngineTransform_22",
      "type": "UnityEngine.Transform",
      "address": 837
    },
    "__intnl_UnityEngineTransform_21": {
      "name": "__intnl_UnityEngineTransform_21",
      "type": "UnityEngine.Transform",
      "address": 831
    },
    "__intnl_UnityEngineTransform_20": {
      "name": "__intnl_UnityEngineTransform_20",
      "type": "UnityEngine.Transform",
      "address": 824
    },
    "__intnl_UnityEngineTransform_27": {
      "name": "__intnl_UnityEngineTransform_27",
      "type": "UnityEngine.Transform",
      "address": 981
    },
    "__intnl_UnityEngineTransform_26": {
      "name": "__intnl_UnityEngineTransform_26",
      "type": "UnityEngine.Transform",
      "address": 885
    },
    "__intnl_UnityEngineTransform_25": {
      "name": "__intnl_UnityEngineTransform_25",
      "type": "UnityEngine.Transform",
      "address": 849
    },
    "__intnl_UnityEngineTransform_24": {
      "name": "__intnl_UnityEngineTransform_24",
      "type": "UnityEngine.Transform",
      "address": 847
    },
    "__intnl_UnityEngineVector3Array_1": {
      "name": "__intnl_UnityEngineVector3Array_1",
      "type": "UnityEngine.Vector3[]",
      "address": 840
    },
    "__intnl_UnityEngineVector3Array_0": {
      "name": "__intnl_UnityEngineVector3Array_0",
      "type": "UnityEngine.Vector3[]",
      "address": 785
    },
    "__intnl_UnityEngineVector3Array_3": {
      "name": "__intnl_UnityEngineVector3Array_3",
      "type": "UnityEngine.Vector3[]",
      "address": 877
    },
    "__intnl_UnityEngineVector3Array_2": {
      "name": "__intnl_UnityEngineVector3Array_2",
      "type": "UnityEngine.Vector3[]",
      "address": 868
    },
    "__lcl_t3_UnityEngineTransform_0": {
      "name": "__lcl_t3_UnityEngineTransform_0",
      "type": "UnityEngine.Transform",
      "address": 650
    },
    "__gintnl_SystemUInt32_95": {
      "name": "__gintnl_SystemUInt32_95",
      "type": "System.UInt32",
      "address": 350
    },
    "__gintnl_SystemUInt32_85": {
      "name": "__gintnl_SystemUInt32_85",
      "type": "System.UInt32",
      "address": 331
    },
    "__gintnl_SystemUInt32_55": {
      "name": "__gintnl_SystemUInt32_55",
      "type": "System.UInt32",
      "address": 285
    },
    "__gintnl_SystemUInt32_45": {
      "name": "__gintnl_SystemUInt32_45",
      "type": "System.UInt32",
      "address": 264
    },
    "__gintnl_SystemUInt32_75": {
      "name": "__gintnl_SystemUInt32_75",
      "type": "System.UInt32",
      "address": 308
    },
    "__gintnl_SystemUInt32_65": {
      "name": "__gintnl_SystemUInt32_65",
      "type": "System.UInt32",
      "address": 295
    },
    "__gintnl_SystemUInt32_15": {
      "name": "__gintnl_SystemUInt32_15",
      "type": "System.UInt32",
      "address": 135
    },
    "__gintnl_SystemUInt32_35": {
      "name": "__gintnl_SystemUInt32_35",
      "type": "System.UInt32",
      "address": 217
    },
    "__gintnl_SystemUInt32_25": {
      "name": "__gintnl_SystemUInt32_25",
      "type": "System.UInt32",
      "address": 178
    },
    "__intnl_SystemUInt32_1": {
      "name": "__intnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 942
    },
    "__const_SystemInt32_9": {
      "name": "__const_SystemInt32_9",
      "type": "System.Int32",
      "address": 342
    },
    "__const_SystemInt32_8": {
      "name": "__const_SystemInt32_8",
      "type": "System.Int32",
      "address": 321
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 84
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 83
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 239
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 85
    },
    "__const_SystemInt32_5": {
      "name": "__const_SystemInt32_5",
      "type": "System.Int32",
      "address": 260
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 243
    },
    "__const_SystemInt32_7": {
      "name": "__const_SystemInt32_7",
      "type": "System.Int32",
      "address": 283
    },
    "__const_SystemInt32_6": {
      "name": "__const_SystemInt32_6",
      "type": "System.Int32",
      "address": 280
    },
    "sensitivity": {
      "name": "sensitivity",
      "type": "System.Single",
      "address": 66
    },
    "__const_SystemInt32_14": {
      "name": "__const_SystemInt32_14",
      "type": "System.Int32",
      "address": 495
    },
    "__1_lineRenderer__param": {
      "name": "__1_lineRenderer__param",
      "type": "UnityEngine.LineRenderer",
      "address": 417
    },
    "__lcl_udonBehaviours_UnityEngineComponentArray_0": {
      "name": "__lcl_udonBehaviours_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 926
    },
    "__0_penIdVector__param": {
      "name": "__0_penIdVector__param",
      "type": "UnityEngine.Vector3",
      "address": 419
    },
    "__intnl_SystemBoolean_8": {
      "name": "__intnl_SystemBoolean_8",
      "type": "System.Boolean",
      "address": 576
    },
    "__intnl_SystemBoolean_9": {
      "name": "__intnl_SystemBoolean_9",
      "type": "System.Boolean",
      "address": 584
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 524
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 526
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 528
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 530
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 531
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 534
    },
    "__intnl_SystemBoolean_6": {
      "name": "__intnl_SystemBoolean_6",
      "type": "System.Boolean",
      "address": 536
    },
    "__intnl_SystemBoolean_7": {
      "name": "__intnl_SystemBoolean_7",
      "type": "System.Boolean",
      "address": 551
    },
    "_objectSync": {
      "name": "_objectSync",
      "type": "VRC.SDK3.Components.VRCObjectSync",
      "address": 27
    },
    "__28__intnlparam": {
      "name": "__28__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 403
    },
    "__intnl_SystemString_4": {
      "name": "__intnl_SystemString_4",
      "type": "System.String",
      "address": 706
    },
    "results4": {
      "name": "results4",
      "type": "UnityEngine.Collider[]",
      "address": 68
    },
    "inkPoolNotSynced": {
      "name": "inkPoolNotSynced",
      "type": "UnityEngine.Transform",
      "address": 10
    },
    "headPos": {
      "name": "headPos",
      "type": "UnityEngine.Vector3",
      "address": 57
    },
    "headRot": {
      "name": "headRot",
      "type": "UnityEngine.Quaternion",
      "address": 59
    },
    "wh": {
      "name": "wh",
      "type": "UnityEngine.Vector2",
      "address": 61
    },
    "__0_get_isUserInVR__ret": {
      "name": "__0_get_isUserInVR__ret",
      "type": "System.Boolean",
      "address": 89
    },
    "surftraceMask": {
      "name": "surftraceMask",
      "type": "System.Int32",
      "address": 70
    },
    "__this_VRCUdonUdonBehaviour_15": {
      "name": "__this_VRCUdonUdonBehaviour_15",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 275
    },
    "__const_SystemType_4": {
      "name": "__const_SystemType_4",
      "type": "System.Type",
      "address": 152
    },
    "__const_SystemString_76": {
      "name": "__const_SystemString_76",
      "type": "System.String",
      "address": 468
    },
    "__const_SystemString_77": {
      "name": "__const_SystemString_77",
      "type": "System.String",
      "address": 469
    },
    "__const_SystemString_74": {
      "name": "__const_SystemString_74",
      "type": "System.String",
      "address": 466
    },
    "__const_SystemString_75": {
      "name": "__const_SystemString_75",
      "type": "System.String",
      "address": 467
    },
    "__const_SystemString_72": {
      "name": "__const_SystemString_72",
      "type": "System.String",
      "address": 460
    },
    "__const_SystemString_73": {
      "name": "__const_SystemString_73",
      "type": "System.String",
      "address": 461
    },
    "__const_SystemString_70": {
      "name": "__const_SystemString_70",
      "type": "System.String",
      "address": 458
    },
    "__const_SystemString_71": {
      "name": "__const_SystemString_71",
      "type": "System.String",
      "address": 459
    },
    "__const_SystemString_78": {
      "name": "__const_SystemString_78",
      "type": "System.String",
      "address": 474
    },
    "__const_SystemString_79": {
      "name": "__const_SystemString_79",
      "type": "System.String",
      "address": 475
    },
    "__intnl_SystemSingle_5": {
      "name": "__intnl_SystemSingle_5",
      "type": "System.Single",
      "address": 523
    },
    "__gintnl_SystemUInt32_158": {
      "name": "__gintnl_SystemUInt32_158",
      "type": "System.UInt32",
      "address": 490
    },
    "__gintnl_SystemUInt32_159": {
      "name": "__gintnl_SystemUInt32_159",
      "type": "System.UInt32",
      "address": 508
    },
    "__gintnl_SystemUInt32_154": {
      "name": "__gintnl_SystemUInt32_154",
      "type": "System.UInt32",
      "address": 476
    },
    "__gintnl_SystemUInt32_155": {
      "name": "__gintnl_SystemUInt32_155",
      "type": "System.UInt32",
      "address": 479
    },
    "__gintnl_SystemUInt32_156": {
      "name": "__gintnl_SystemUInt32_156",
      "type": "System.UInt32",
      "address": 481
    },
    "__gintnl_SystemUInt32_157": {
      "name": "__gintnl_SystemUInt32_157",
      "type": "System.UInt32",
      "address": 486
    },
    "__gintnl_SystemUInt32_150": {
      "name": "__gintnl_SystemUInt32_150",
      "type": "System.UInt32",
      "address": 463
    },
    "__gintnl_SystemUInt32_151": {
      "name": "__gintnl_SystemUInt32_151",
      "type": "System.UInt32",
      "address": 464
    },
    "__gintnl_SystemUInt32_152": {
      "name": "__gintnl_SystemUInt32_152",
      "type": "System.UInt32",
      "address": 465
    },
    "__gintnl_SystemUInt32_153": {
      "name": "__gintnl_SystemUInt32_153",
      "type": "System.UInt32",
      "address": 470
    },
    "marker": {
      "name": "marker",
      "type": "UnityEngine.Renderer",
      "address": 21
    },
    "__0_trailRenderer__param": {
      "name": "__0_trailRenderer__param",
      "type": "UnityEngine.TrailRenderer",
      "address": 317
    },
    "__intnl_SystemObject_5": {
      "name": "__intnl_SystemObject_5",
      "type": "System.Object",
      "address": 570
    },
    "__const_SystemSingle_10": {
      "name": "__const_SystemSingle_10",
      "type": "System.Single",
      "address": 220
    },
    "__const_SystemSingle_11": {
      "name": "__const_SystemSingle_11",
      "type": "System.Single",
      "address": 238
    },
    "__const_SystemSingle_12": {
      "name": "__const_SystemSingle_12",
      "type": "System.Single",
      "address": 424
    },
    "__const_SystemSingle_13": {
      "name": "__const_SystemSingle_13",
      "type": "System.Single",
      "address": 444
    },
    "__const_SystemSingle_14": {
      "name": "__const_SystemSingle_14",
      "type": "System.Single",
      "address": 511
    },
    "__21__intnlparam": {
      "name": "__21__intnlparam",
      "type": "UnityEngine.Vector3",
      "address": 391
    },
    "__intnl_SystemObject_8": {
      "name": "__intnl_SystemObject_8",
      "type": "System.Object",
      "address": 579
    },
    "__lcl_penIdVector_UnityEngineVector3_0": {
      "name": "__lcl_penIdVector_UnityEngineVector3_0",
      "type": "UnityEngine.Vector3",
      "address": 809
    },
    "__intnl_VRCUdonUdonBehaviour_0": {
      "name": "__intnl_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 550
    },
    "__const_UnityEngineVector2_0": {
      "name": "__const_UnityEngineVector2_0",
      "type": "UnityEngine.Vector2",
      "address": 191
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 131
    },
    "inkPool": {
      "name": "inkPool",
      "type": "UnityEngine.Transform",
      "address": 8
    },
    "__this_VRCUdonUdonBehaviour_9": {
      "name": "__this_VRCUdonUdonBehaviour_9",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 241
    },
    "__this_VRCUdonUdonBehaviour_8": {
      "name": "__this_VRCUdonUdonBehaviour_8",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 234
    },
    "__this_VRCUdonUdonBehaviour_3": {
      "name": "__this_VRCUdonUdonBehaviour_3",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 142
    },
    "__this_VRCUdonUdonBehaviour_2": {
      "name": "__this_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 138
    },
    "__this_VRCUdonUdonBehaviour_1": {
      "name": "__this_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 98
    },
    "__this_VRCUdonUdonBehaviour_0": {
      "name": "__this_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 95
    },
    "__this_VRCUdonUdonBehaviour_7": {
      "name": "__this_VRCUdonUdonBehaviour_7",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 231
    },
    "__this_VRCUdonUdonBehaviour_6": {
      "name": "__this_VRCUdonUdonBehaviour_6",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 224
    },
    "__this_VRCUdonUdonBehaviour_5": {
      "name": "__this_VRCUdonUdonBehaviour_5",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 223
    },
    "__this_VRCUdonUdonBehaviour_4": {
      "name": "__this_VRCUdonUdonBehaviour_4",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 197
    },
    "__intnl_SystemBoolean_111": {
      "name": "__intnl_SystemBoolean_111",
      "type": "System.Boolean",
      "address": 997
    },
    "__intnl_SystemBoolean_110": {
      "name": "__intnl_SystemBoolean_110",
      "type": "System.Boolean",
      "address": 994
    },
    "__intnl_SystemBoolean_112": {
      "name": "__intnl_SystemBoolean_112",
      "type": "System.Boolean",
      "address": 1000
    },
    "__intnl_UnityEngineGameObject_16": {
      "name": "__intnl_UnityEngineGameObject_16",
      "type": "UnityEngine.GameObject",
      "address": 789
    },
    "__intnl_UnityEngineTransform_3": {
      "name": "__intnl_UnityEngineTransform_3",
      "type": "UnityEngine.Transform",
      "address": 556
    },
    "__6_data__param": {
      "name": "__6_data__param",
      "type": "UnityEngine.Vector3[]",
      "address": 373
    },
    "inkWidth": {
      "name": "inkWidth",
      "type": "System.Single",
      "address": 45
    },
    "_localPlayerIdVector": {
      "name": "_localPlayerIdVector",
      "type": "UnityEngine.Vector3",
      "address": 52
    },
    "__intnl_SystemDouble_6": {
      "name": "__intnl_SystemDouble_6",
      "type": "System.Double",
      "address": 822
    },
    "__lcl_line_UnityEngineLineRenderer_0": {
      "name": "__lcl_line_UnityEngineLineRenderer_0",
      "type": "UnityEngine.LineRenderer",
      "address": 841
    },
    "__const_SystemString_8": {
      "name": "__const_SystemString_8",
      "type": "System.String",
      "address": 140
    },
    "__intnl_SystemBoolean_88": {
      "name": "__intnl_SystemBoolean_88",
      "type": "System.Boolean",
      "address": 888
    },
    "__intnl_SystemBoolean_98": {
      "name": "__intnl_SystemBoolean_98",
      "type": "System.Boolean",
      "address": 937
    },
    "__intnl_SystemBoolean_18": {
      "name": "__intnl_SystemBoolean_18",
      "type": "System.Boolean",
      "address": 599
    },
    "__intnl_SystemBoolean_28": {
      "name": "__intnl_SystemBoolean_28",
      "type": "System.Boolean",
      "address": 656
    },
    "__intnl_SystemBoolean_38": {
      "name": "__intnl_SystemBoolean_38",
      "type": "System.Boolean",
      "address": 691
    },
    "__intnl_SystemBoolean_48": {
      "name": "__intnl_SystemBoolean_48",
      "type": "System.Boolean",
      "address": 717
    },
    "__intnl_SystemBoolean_58": {
      "name": "__intnl_SystemBoolean_58",
      "type": "System.Boolean",
      "address": 729
    },
    "__intnl_SystemBoolean_68": {
      "name": "__intnl_SystemBoolean_68",
      "type": "System.Boolean",
      "address": 767
    },
    "__intnl_SystemBoolean_78": {
      "name": "__intnl_SystemBoolean_78",
      "type": "System.Boolean",
      "address": 804
    },
    "__intnl_UnityEngineTransform_8": {
      "name": "__intnl_UnityEngineTransform_8",
      "type": "UnityEngine.Transform",
      "address": 605
    },
    "__intnl_UnityEngineVector3_3": {
      "name": "__intnl_UnityEngineVector3_3",
      "type": "UnityEngine.Vector3",
      "address": 591
    },
    "inkPositionChild": {
      "name": "inkPositionChild",
      "type": "UnityEngine.Transform",
      "address": 6
    },
    "__lcl_data_UnityEngineVector3Array_2": {
      "name": "__lcl_data_UnityEngineVector3Array_2",
      "type": "UnityEngine.Vector3[]",
      "address": 770
    },
    "__2_inkId__param": {
      "name": "__2_inkId__param",
      "type": "System.Int32",
      "address": 430
    }
  },
  "entryPoints": [
    {
      "name": "get_AllowCallPen",
      "address": 0
    },
    {
      "name": "get_IsUser",
      "address": 876
    },
    {
      "name": "get_penIdVector",
      "address": 1260
    },
    {
      "name": "__0__Init",
      "address": 2168
    },
    {
      "name": "_UpdateInkData",
      "address": 4344
    },
    {
      "name": "__0__CheckId",
      "address": 5732
    },
    {
      "name": "_update",
      "address": 5828
    },
    {
      "name": "_lateUpdate",
      "address": 7432
    },
    {
      "name": "_postLateUpdate",
      "address": 9376
    },
    {
      "name": "_onTriggerEnter",
      "address": 10580
    },
    {
      "name": "_onPickup",
      "address": 11712
    },
    {
      "name": "_onDrop",
      "address": 12020
    },
    {
      "name": "_onPickupUseDown",
      "address": 12252
    },
    {
      "name": "_onPickupUseUp",
      "address": 13364
    },
    {
      "name": "__0__SetUseDoubleClick",
      "address": 13968
    },
    {
      "name": "__0__SetEnabledLateSync",
      "address": 14064
    },
    {
      "name": "__0__SetUseSurftraceMode",
      "address": 14112
    },
    {
      "name": "_onEnable",
      "address": 14208
    },
    {
      "name": "_onDisable",
      "address": 14324
    },
    {
      "name": "_onDestroy",
      "address": 14440
    },
    {
      "name": "ChangeStateToPenIdle",
      "address": 14584
    },
    {
      "name": "ChangeStateToPenUsing",
      "address": 14864
    },
    {
      "name": "ChangeStateToEraseIdle",
      "address": 15176
    },
    {
      "name": "ChangeStateToEraseUsing",
      "address": 15456
    },
    {
      "name": "_TakeOwnership",
      "address": 15768
    },
    {
      "name": "get_isHeld",
      "address": 15968
    },
    {
      "name": "_Respawn",
      "address": 16016
    },
    {
      "name": "_Clear",
      "address": 16296
    },
    {
      "name": "_EraseOwnInk",
      "address": 16400
    },
    {
      "name": "_UndoDraw",
      "address": 16460
    },
    {
      "name": "__0__PackData",
      "address": 18344
    },
    {
      "name": "__0__SendData",
      "address": 19632
    },
    {
      "name": "__0__UnpackData",
      "address": 20128
    },
    {
      "name": "__0__EraseAbandonedInk",
      "address": 20604
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 1559466544549460938
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.QvPen_Pen"
      }
    },
    "2": {
      "address": 2,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "3": {
      "address": 3,
      "type": "UnityEngine.TrailRenderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "4": {
      "address": 4,
      "type": "UnityEngine.LineRenderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "5": {
      "address": 5,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "6": {
      "address": 6,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "7": {
      "address": 7,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "8": {
      "address": 8,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "9": {
      "address": 9,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "10": {
      "address": 10,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "11": {
      "address": 11,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "12": {
      "address": 12,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "13": {
      "address": 13,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "14": {
      "address": 14,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "15": {
      "address": 15,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "16": {
      "address": 16,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "17": {
      "address": 17,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 3.0
      }
    },
    "18": {
      "address": 18,
      "type": "UnityEngine.Material",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "19": {
      "address": 19,
      "type": "UnityEngine.Material",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "20": {
      "address": 20,
      "type": "UnityEngine.Canvas",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "21": {
      "address": 21,
      "type": "UnityEngine.Renderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "22": {
      "address": 22,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "23": {
      "address": 23,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "24": {
      "address": 24,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "25": {
      "address": 25,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "26": {
      "address": 26,
      "type": "VRC.SDK3.Components.VRCPickup",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "27": {
      "address": 27,
      "type": "VRC.SDK3.Components.VRCObjectSync",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "28": {
      "address": 28,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "29": {
      "address": 29,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Respawn"
      }
    },
    "30": {
      "address": 30,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "31": {
      "address": 31,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "32": {
      "address": 32,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "33": {
      "address": 33,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "34": {
      "address": 34,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "35": {
      "address": 35,
      "type": "UnityEngine.Renderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "36": {
      "address": 36,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "37": {
      "address": 37,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "38": {
      "address": 38,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.01
      }
    },
    "39": {
      "address": 39,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "40": {
      "address": 40,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "41": {
      "address": 41,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "42": {
      "address": 42,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "43": {
      "address": 43,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "44": {
      "address": 44,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "45": {
      "address": 45,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "46": {
      "address": 46,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "47": {
      "address": 47,
      "type": "UnityEngine.MaterialPropertyBlock",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "48": {
      "address": 48,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "49": {
      "address": 49,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "50": {
      "address": 50,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "51": {
      "address": 51,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "52": {
      "address": 52,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "53": {
      "address": 53,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "54": {
      "address": 54,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "55": {
      "address": 55,
      "type": "VRC.SDK3.Data.DataList",
      "value": {
        "isSerializable": true,
        "value": []
      }
    },
    "56": {
      "address": 56,
      "type": "VRC.SDKBase.VRCPlayerApi+TrackingData",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDKBase.VRCPlayerApi+TrackingData",
          "toString": "VRC.SDKBase.VRCPlayerApi+TrackingData"
        }
      }
    },
    "57": {
      "address": 57,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "58": {
      "address": 58,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "59": {
      "address": 59,
      "type": "UnityEngine.Quaternion",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Quaternion",
          "toString": "(0.00000, 0.00000, 0.00000, 0.00000)"
        }
      }
    },
    "60": {
      "address": 60,
      "type": "UnityEngine.Vector2",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector2",
          "toString": "(0.00, 0.00)"
        }
      }
    },
    "61": {
      "address": 61,
      "type": "UnityEngine.Vector2",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector2",
          "toString": "(0.00, 0.00)"
        }
      }
    },
    "62": {
      "address": 62,
      "type": "UnityEngine.Vector2",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector2",
          "toString": "(0.00, 0.00)"
        }
      }
    },
    "63": {
      "address": 63,
      "type": "UnityEngine.Vector2",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector2",
          "toString": "(0.00, 0.00)"
        }
      }
    },
    "64": {
      "address": 64,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "65": {
      "address": 65,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "66": {
      "address": 66,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.75
      }
    },
    "67": {
      "address": 67,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "68": {
      "address": 68,
      "type": "UnityEngine.Collider[]",
      "value": {
        "isSerializable": true,
        "value": [
          null,
          null,
          null,
          null
        ]
      }
    },
    "69": {
      "address": 69,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "70": {
      "address": 70,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": -1
      }
    },
    "71": {
      "address": 71,
      "type": "UnityEngine.Collider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "72": {
      "address": 72,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "73": {
      "address": 73,
      "type": "UnityEngine.Collider[]",
      "value": {
        "isSerializable": true,
        "value": [
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null,
          null
        ]
      }
    },
    "74": {
      "address": 74,
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.949, 0.490, 0.290, 1.000)"
        }
      }
    },
    "75": {
      "address": 75,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "76": {
      "address": 76,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "77": {
      "address": 77,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "78": {
      "address": 78,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "79": {
      "address": 79,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "UnityEngine.SphereCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "80": {
      "address": 80,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "81": {
      "address": 81,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.01
      }
    },
    "82": {
      "address": 82,
      "type": "System.Single[]",
      "value": {
        "isSerializable": true,
        "value": [
          0.0,
          0.0,
          0.0
        ]
      }
    },
    "83": {
      "address": 83,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "84": {
      "address": 84,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "85": {
      "address": 85,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "86": {
      "address": 86,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "87": {
      "address": 87,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "88": {
      "address": 88,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 608
      }
    },
    "89": {
      "address": 89,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "90": {
      "address": 90,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 1.0
      }
    },
    "91": {
      "address": 91,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "92": {
      "address": 92,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "93": {
      "address": 93,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "94": {
      "address": 94,
      "type": "VRC.SDK3.Components.VRCPickup",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "95": {
      "address": 95,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "96": {
      "address": 96,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "VRC.SDK3.Components.VRCPickup, VRCSDK3, Version=1.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "97": {
      "address": 97,
      "type": "VRC.SDK3.Components.VRCObjectSync",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "98": {
      "address": 98,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "99": {
      "address": 99,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "VRC.SDK3.Components.VRCObjectSync, VRCSDK3, Version=1.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "100": {
      "address": 100,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "101": {
      "address": 101,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "102": {
      "address": 102,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "103": {
      "address": 103,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "104": {
      "address": 104,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "105": {
      "address": 105,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1576
      }
    },
    "106": {
      "address": 106,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1632
      }
    },
    "107": {
      "address": 107,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "108": {
      "address": 108,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1840
      }
    },
    "109": {
      "address": 109,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "110": {
      "address": 110,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "111": {
      "address": 111,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1812
      }
    },
    "112": {
      "address": 112,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2008
      }
    },
    "113": {
      "address": 113,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2084
      }
    },
    "114": {
      "address": 114,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "115": {
      "address": 115,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2212
      }
    },
    "116": {
      "address": 116,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "/{0}"
      }
    },
    "117": {
      "address": 117,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen_Objects"
      }
    },
    "118": {
      "address": 118,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2488
      }
    },
    "119": {
      "address": 119,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "120": {
      "address": 120,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "121": {
      "address": 121,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2632
      }
    },
    "122": {
      "address": 122,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.GameObject, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "123": {
      "address": 123,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2820
      }
    },
    "124": {
      "address": 124,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2792
      }
    },
    "125": {
      "address": 125,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "126": {
      "address": 126,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "127": {
      "address": 127,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "0x{0:x2}{1:x3}{2:x3}"
      }
    },
    "128": {
      "address": 128,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2836
      }
    },
    "129": {
      "address": 129,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2948
      }
    },
    "130": {
      "address": 130,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3060
      }
    },
    "131": {
      "address": 131,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "{0} ({1})"
      }
    },
    "132": {
      "address": 132,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "InkPool"
      }
    },
    "133": {
      "address": 133,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_AllowCallPen"
      }
    },
    "134": {
      "address": 134,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_AllowCallPen__ret"
      }
    },
    "135": {
      "address": 135,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3444
      }
    },
    "136": {
      "address": 136,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "137": {
      "address": 137,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "138": {
      "address": 138,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "139": {
      "address": 139,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_penId__param"
      }
    },
    "140": {
      "address": 140,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_pen__param"
      }
    },
    "141": {
      "address": 141,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_Register"
      }
    },
    "142": {
      "address": 142,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "143": {
      "address": 143,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__RegisterPen"
      }
    },
    "144": {
      "address": 144,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_InkPoolSynced"
      }
    },
    "145": {
      "address": 145,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_InkPoolSynced__ret"
      }
    },
    "146": {
      "address": 146,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_InkPoolNotSynced"
      }
    },
    "147": {
      "address": 147,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_InkPoolNotSynced__ret"
      }
    },
    "148": {
      "address": 148,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3876
      }
    },
    "149": {
      "address": 149,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen"
      }
    },
    "150": {
      "address": 150,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3916
      }
    },
    "151": {
      "address": 151,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Draw"
      }
    },
    "152": {
      "address": 152,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "UnityEngine.Renderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "153": {
      "address": 153,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4108
      }
    },
    "154": {
      "address": 154,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(1.00, 1.00, 1.00)"
        }
      }
    },
    "155": {
      "address": 155,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4260
      }
    },
    "156": {
      "address": 156,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.005
      }
    },
    "157": {
      "address": 157,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.001
      }
    },
    "158": {
      "address": 158,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "inkWidth"
      }
    },
    "159": {
      "address": 159,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "inkMeshLayer"
      }
    },
    "160": {
      "address": 160,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "inkColliderLayer"
      }
    },
    "161": {
      "address": 161,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4664
      }
    },
    "162": {
      "address": 162,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "pcInkMaterial"
      }
    },
    "163": {
      "address": 163,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_roundedTrailShader"
      }
    },
    "164": {
      "address": 164,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_roundedTrailShader__ret"
      }
    },
    "165": {
      "address": 165,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "rounded_trail"
      }
    },
    "166": {
      "address": 166,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "167": {
      "address": 167,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_Width"
      }
    },
    "168": {
      "address": 168,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "colorGradient"
      }
    },
    "169": {
      "address": 169,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "surftraceMask"
      }
    },
    "170": {
      "address": 170,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "171": {
      "address": 171,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "172": {
      "address": 172,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 5776
      }
    },
    "173": {
      "address": 173,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 5852
      }
    },
    "174": {
      "address": 174,
      "type": "UnityEngine.KeyCode",
      "value": {
        "isSerializable": true,
        "value": 9
      }
    },
    "175": {
      "address": 175,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6012
      }
    },
    "176": {
      "address": 176,
      "type": "UnityEngine.KeyCode",
      "value": {
        "isSerializable": true,
        "value": 8
      }
    },
    "177": {
      "address": 177,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6128
      }
    },
    "178": {
      "address": 178,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6192
      }
    },
    "179": {
      "address": 179,
      "type": "UnityEngine.KeyCode",
      "value": {
        "isSerializable": true,
        "value": 127
      }
    },
    "180": {
      "address": 180,
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "181": {
      "address": 181,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Clear"
      }
    },
    "182": {
      "address": 182,
      "type": "UnityEngine.KeyCode",
      "value": {
        "isSerializable": true,
        "value": 278
      }
    },
    "183": {
      "address": 183,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Respawn"
      }
    },
    "184": {
      "address": 184,
      "type": "UnityEngine.KeyCode",
      "value": {
        "isSerializable": true,
        "value": 273
      }
    },
    "185": {
      "address": 185,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 5.0
      }
    },
    "186": {
      "address": 186,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6572
      }
    },
    "187": {
      "address": 187,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "188": {
      "address": 188,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Sensitivity -> {0:f3}"
      }
    },
    "189": {
      "address": 189,
      "type": "UnityEngine.KeyCode",
      "value": {
        "isSerializable": true,
        "value": 274
      }
    },
    "190": {
      "address": 190,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 6752
      }
    },
    "191": {
      "address": 191,
      "type": "UnityEngine.Vector2",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector2",
          "toString": "(0.00, 0.00)"
        }
      }
    },
    "192": {
      "address": 192,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "UnityEngine.RectTransform, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "193": {
      "address": 193,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 3763.2002
      }
    },
    "194": {
      "address": 194,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 2160.0
      }
    },
    "195": {
      "address": 195,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 7196
      }
    },
    "196": {
      "address": 196,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "197": {
      "address": 197,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "198": {
      "address": 198,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ChangeStateToPenIdle"
      }
    },
    "199": {
      "address": 199,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "200": {
      "address": 200,
      "type": "UnityEngine.Quaternion",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Quaternion",
          "toString": "(0.00000, 0.00000, 0.00000, 1.00000)"
        }
      }
    },
    "201": {
      "address": 201,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 7456
      }
    },
    "202": {
      "address": 202,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "203": {
      "address": 203,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 7516
      }
    },
    "204": {
      "address": 204,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 7668
      }
    },
    "205": {
      "address": 205,
      "type": "VRC.SDKBase.VRCPlayerApi+TrackingDataType",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "206": {
      "address": 206,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 1.00)"
        }
      }
    },
    "207": {
      "address": 207,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.Transform, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "208": {
      "address": 208,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Mouse X"
      }
    },
    "209": {
      "address": 209,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Mouse Y"
      }
    },
    "210": {
      "address": 210,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 8564
      }
    },
    "211": {
      "address": 211,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 1.999
      }
    },
    "212": {
      "address": 212,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 8876
      }
    },
    "213": {
      "address": 213,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 32.0
      }
    },
    "214": {
      "address": 214,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 9504
      }
    },
    "215": {
      "address": 215,
      "type": "UnityEngine.QueryTriggerInteraction",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "216": {
      "address": 216,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "UnityEngine.LineRenderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "217": {
      "address": 217,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 10416
      }
    },
    "218": {
      "address": 218,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "219": {
      "address": 219,
      "type": "UnityEngine.Collider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "220": {
      "address": 220,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.05
      }
    },
    "221": {
      "address": 221,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 11356
      }
    },
    "222": {
      "address": 222,
      "type": "UnityEngine.Collider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "223": {
      "address": 223,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "224": {
      "address": 224,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "225": {
      "address": 225,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__1_pen__param"
      }
    },
    "226": {
      "address": 226,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_SetLastUsedPen"
      }
    },
    "227": {
      "address": 227,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "OnPenPickup"
      }
    },
    "228": {
      "address": 228,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_TakeOwnership"
      }
    },
    "229": {
      "address": 229,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__TakeOwnership__ret"
      }
    },
    "230": {
      "address": 230,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "StartUsing"
      }
    },
    "231": {
      "address": 231,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "232": {
      "address": 232,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "OnPenDrop"
      }
    },
    "233": {
      "address": 233,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "EndUsing"
      }
    },
    "234": {
      "address": 234,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "235": {
      "address": 235,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_ClearSyncBuffer"
      }
    },
    "236": {
      "address": 236,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 12216
      }
    },
    "237": {
      "address": 237,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 12232
      }
    },
    "238": {
      "address": 238,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.2
      }
    },
    "239": {
      "address": 239,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "240": {
      "address": 240,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 12704
      }
    },
    "241": {
      "address": 241,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "242": {
      "address": 242,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ChangeStateToEraseIdle"
      }
    },
    "243": {
      "address": 243,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "244": {
      "address": 244,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "245": {
      "address": 245,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 12952
      }
    },
    "246": {
      "address": 246,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "247": {
      "address": 247,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Unexpected state : {0} at {1} Double Clicked"
      }
    },
    "248": {
      "address": 248,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 12884
      }
    },
    "249": {
      "address": 249,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "250": {
      "address": 250,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "251": {
      "address": 251,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "OnPickupUseDown"
      }
    },
    "252": {
      "address": 252,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "253": {
      "address": 253,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ChangeStateToPenUsing"
      }
    },
    "254": {
      "address": 254,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "255": {
      "address": 255,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ChangeStateToEraseUsing"
      }
    },
    "256": {
      "address": 256,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13200
      }
    },
    "257": {
      "address": 257,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13328
      }
    },
    "258": {
      "address": 258,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Unexpected state : {0} at {1}"
      }
    },
    "259": {
      "address": 259,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13260
      }
    },
    "260": {
      "address": 260,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "261": {
      "address": 261,
      "type": "System.UInt32[]",
      "value": {
        "isSerializable": true,
        "value": [
          13588,
          13508,
          13708,
          13548
        ]
      }
    },
    "262": {
      "address": 262,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "263": {
      "address": 263,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "264": {
      "address": 264,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13700
      }
    },
    "265": {
      "address": 265,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Change state : {0} to {1}"
      }
    },
    "266": {
      "address": 266,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "EraserIdle"
      }
    },
    "267": {
      "address": 267,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13632
      }
    },
    "268": {
      "address": 268,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13820
      }
    },
    "269": {
      "address": 269,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "PenIdle"
      }
    },
    "270": {
      "address": 270,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13752
      }
    },
    "271": {
      "address": 271,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13940
      }
    },
    "272": {
      "address": 272,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 13872
      }
    },
    "273": {
      "address": 273,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "OnPickupUseUp"
      }
    },
    "274": {
      "address": 274,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "275": {
      "address": 275,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "276": {
      "address": 276,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "277": {
      "address": 277,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "278": {
      "address": 278,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "279": {
      "address": 279,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 14464
      }
    },
    "280": {
      "address": 280,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "281": {
      "address": 281,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 14656
      }
    },
    "282": {
      "address": 282,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 14728
      }
    },
    "283": {
      "address": 283,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "284": {
      "address": 284,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 14800
      }
    },
    "285": {
      "address": 285,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 14816
      }
    },
    "286": {
      "address": 286,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 14936
      }
    },
    "287": {
      "address": 287,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15008
      }
    },
    "288": {
      "address": 288,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15024
      }
    },
    "289": {
      "address": 289,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15096
      }
    },
    "290": {
      "address": 290,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15112
      }
    },
    "291": {
      "address": 291,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15128
      }
    },
    "292": {
      "address": 292,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15248
      }
    },
    "293": {
      "address": 293,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15320
      }
    },
    "294": {
      "address": 294,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15336
      }
    },
    "295": {
      "address": 295,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15408
      }
    },
    "296": {
      "address": 296,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15528
      }
    },
    "297": {
      "address": 297,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15544
      }
    },
    "298": {
      "address": 298,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15616
      }
    },
    "299": {
      "address": 299,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15632
      }
    },
    "300": {
      "address": 300,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15648
      }
    },
    "301": {
      "address": 301,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 15720
      }
    },
    "302": {
      "address": 302,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "303": {
      "address": 303,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16040
      }
    },
    "304": {
      "address": 304,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16112
      }
    },
    "305": {
      "address": 305,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16168
      }
    },
    "306": {
      "address": 306,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__5_penId__param"
      }
    },
    "307": {
      "address": 307,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_Clear"
      }
    },
    "308": {
      "address": 308,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16424
      }
    },
    "309": {
      "address": 309,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16440
      }
    },
    "310": {
      "address": 310,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16484
      }
    },
    "311": {
      "address": 311,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16500
      }
    },
    "312": {
      "address": 312,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "get_InkId"
      }
    },
    "313": {
      "address": 313,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_get_InkId__ret"
      }
    },
    "314": {
      "address": 314,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16776
      }
    },
    "315": {
      "address": 315,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16928
      }
    },
    "316": {
      "address": 316,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "317": {
      "address": 317,
      "type": "UnityEngine.TrailRenderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "318": {
      "address": 318,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "319": {
      "address": 319,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "320": {
      "address": 320,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "321": {
      "address": 321,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "322": {
      "address": 322,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16840
      }
    },
    "323": {
      "address": 323,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 16984
      }
    },
    "324": {
      "address": 324,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "325": {
      "address": 325,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_IncrementInkId"
      }
    },
    "326": {
      "address": 326,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 17064
      }
    },
    "327": {
      "address": 327,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "328": {
      "address": 328,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 17464
      }
    },
    "329": {
      "address": 329,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "330": {
      "address": 330,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "331": {
      "address": 331,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 17812
      }
    },
    "332": {
      "address": 332,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "333": {
      "address": 333,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "334": {
      "address": 334,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "335": {
      "address": 335,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 17596
      }
    },
    "336": {
      "address": 336,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 17680
      }
    },
    "337": {
      "address": 337,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 17904
      }
    },
    "338": {
      "address": 338,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 17836
      }
    },
    "339": {
      "address": 339,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 17980
      }
    },
    "340": {
      "address": 340,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 18056
      }
    },
    "341": {
      "address": 341,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 18284
      }
    },
    "342": {
      "address": 342,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 4
      }
    },
    "343": {
      "address": 343,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "344": {
      "address": 344,
      "type": "UnityEngine.LineRenderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "345": {
      "address": 345,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "346": {
      "address": 346,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "347": {
      "address": 347,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "348": {
      "address": 348,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 18644
      }
    },
    "349": {
      "address": 349,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 19124
      }
    },
    "350": {
      "address": 350,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 18936
      }
    },
    "351": {
      "address": 351,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 18992
      }
    },
    "352": {
      "address": 352,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 19216
      }
    },
    "353": {
      "address": 353,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 19148
      }
    },
    "354": {
      "address": 354,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 19292
      }
    },
    "355": {
      "address": 355,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 19368
      }
    },
    "356": {
      "address": 356,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 19572
      }
    },
    "357": {
      "address": 357,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_data__param"
      }
    },
    "358": {
      "address": 358,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0__SendData"
      }
    },
    "359": {
      "address": 359,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 19984
      }
    },
    "360": {
      "address": 360,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "361": {
      "address": 361,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "362": {
      "address": 362,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 20172
      }
    },
    "363": {
      "address": 363,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "364": {
      "address": 364,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "365": {
      "address": 365,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 20392
      }
    },
    "366": {
      "address": 366,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "367": {
      "address": 367,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 3
      }
    },
    "368": {
      "address": 368,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 20484
      }
    },
    "369": {
      "address": 369,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "370": {
      "address": 370,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 4
      }
    },
    "371": {
      "address": 371,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 20576
      }
    },
    "372": {
      "address": 372,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "373": {
      "address": 373,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "374": {
      "address": 374,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 20648
      }
    },
    "375": {
      "address": 375,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 20772
      }
    },
    "376": {
      "address": 376,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1024
      }
    },
    "377": {
      "address": 377,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "378": {
      "address": 378,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "379": {
      "address": 379,
      "type": "VRC.SDK3.Data.TokenType",
      "value": {
        "isSerializable": true,
        "value": 6
      }
    },
    "380": {
      "address": 380,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__1_penId__param"
      }
    },
    "381": {
      "address": 381,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_inkId__param"
      }
    },
    "382": {
      "address": 382,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_HasInk"
      }
    },
    "383": {
      "address": 383,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0___0_HasInk__ret"
      }
    },
    "384": {
      "address": 384,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 21640
      }
    },
    "385": {
      "address": 385,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "386": {
      "address": 386,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "387": {
      "address": 387,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "388": {
      "address": 388,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 21716
      }
    },
    "389": {
      "address": 389,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 21772
      }
    },
    "390": {
      "address": 390,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "391": {
      "address": 391,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "392": {
      "address": 392,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 21828
      }
    },
    "393": {
      "address": 393,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 22080
      }
    },
    "394": {
      "address": 394,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 22160
      }
    },
    "395": {
      "address": 395,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "396": {
      "address": 396,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "397": {
      "address": 397,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Ink"
      }
    },
    "398": {
      "address": 398,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 22340
      }
    },
    "399": {
      "address": 399,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "400": {
      "address": 400,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "401": {
      "address": 401,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "402": {
      "address": 402,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "403": {
      "address": 403,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "404": {
      "address": 404,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 22440
      }
    },
    "405": {
      "address": 405,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "406": {
      "address": 406,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Failed TrySetIdFromInk pen: {0}, ink: {1}"
      }
    },
    "407": {
      "address": 407,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__2_penId__param"
      }
    },
    "408": {
      "address": 408,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__1_inkId__param"
      }
    },
    "409": {
      "address": 409,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_inkInstance__param"
      }
    },
    "410": {
      "address": 410,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_SetInk"
      }
    },
    "411": {
      "address": 411,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 22672
      }
    },
    "412": {
      "address": 412,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 23292
      }
    },
    "413": {
      "address": 413,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 23372
      }
    },
    "414": {
      "address": 414,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "415": {
      "address": 415,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "416": {
      "address": 416,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 23752
      }
    },
    "417": {
      "address": 417,
      "type": "UnityEngine.LineRenderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "418": {
      "address": 418,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "InkCollider"
      }
    },
    "419": {
      "address": 419,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "420": {
      "address": 420,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "421": {
      "address": 421,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24204
      }
    },
    "422": {
      "address": 422,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24444
      }
    },
    "423": {
      "address": 423,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24252
      }
    },
    "424": {
      "address": 424,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 3.0
      }
    },
    "425": {
      "address": 425,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24312
      }
    },
    "426": {
      "address": 426,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24520
      }
    },
    "427": {
      "address": 427,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24596
      }
    },
    "428": {
      "address": 428,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24632
      }
    },
    "429": {
      "address": 429,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "430": {
      "address": 430,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "431": {
      "address": 431,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24808
      }
    },
    "432": {
      "address": 432,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24704
      }
    },
    "433": {
      "address": 433,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24760
      }
    },
    "434": {
      "address": 434,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 24972
      }
    },
    "435": {
      "address": 435,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "436": {
      "address": 436,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "437": {
      "address": 437,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "438": {
      "address": 438,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "439": {
      "address": 439,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "440": {
      "address": 440,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25124
      }
    },
    "441": {
      "address": 441,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25188
      }
    },
    "442": {
      "address": 442,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25428
      }
    },
    "443": {
      "address": 443,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25236
      }
    },
    "444": {
      "address": 444,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 4.0
      }
    },
    "445": {
      "address": 445,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25296
      }
    },
    "446": {
      "address": 446,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25520
      }
    },
    "447": {
      "address": 447,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25452
      }
    },
    "448": {
      "address": 448,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25612
      }
    },
    "449": {
      "address": 449,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25544
      }
    },
    "450": {
      "address": 450,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25648
      }
    },
    "451": {
      "address": 451,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25712
      }
    },
    "452": {
      "address": 452,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25832
      }
    },
    "453": {
      "address": 453,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 25940
      }
    },
    "454": {
      "address": 454,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26064
      }
    },
    "455": {
      "address": 455,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26140
      }
    },
    "456": {
      "address": 456,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26196
      }
    },
    "457": {
      "address": 457,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26252
      }
    },
    "458": {
      "address": 458,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__3_penId__param"
      }
    },
    "459": {
      "address": 459,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__2_inkId__param"
      }
    },
    "460": {
      "address": 460,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_RemoveInk"
      }
    },
    "461": {
      "address": 461,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0___0_RemoveInk__ret"
      }
    },
    "462": {
      "address": 462,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26520
      }
    },
    "463": {
      "address": 463,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26644
      }
    },
    "464": {
      "address": 464,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26716
      }
    },
    "465": {
      "address": 465,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26688
      }
    },
    "466": {
      "address": 466,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__4_penId__param"
      }
    },
    "467": {
      "address": 467,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_ownerIdVector__param"
      }
    },
    "468": {
      "address": 468,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_RemoveUserInk"
      }
    },
    "469": {
      "address": 469,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0___0_RemoveUserInk__ret"
      }
    },
    "470": {
      "address": 470,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 26964
      }
    },
    "471": {
      "address": 471,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": -1
      }
    },
    "472": {
      "address": 472,
      "type": "UnityEngine.QueryTriggerInteraction",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "473": {
      "address": 473,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
      }
    },
    "474": {
      "address": 474,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_interact"
      }
    },
    "475": {
      "address": 475,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "{0}{1}"
      }
    },
    "476": {
      "address": 476,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 27696
      }
    },
    "477": {
      "address": 477,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "478": {
      "address": 478,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "479": {
      "address": 479,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 27824
      }
    },
    "480": {
      "address": 480,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "481": {
      "address": 481,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 27952
      }
    },
    "482": {
      "address": 482,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "483": {
      "address": 483,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "484": {
      "address": 484,
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.000, 0.000, 0.000, 0.000)"
        }
      }
    },
    "485": {
      "address": 485,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "<color=\"#{0}\">"
      }
    },
    "486": {
      "address": 486,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 28100
      }
    },
    "487": {
      "address": 487,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "488": {
      "address": 488,
      "type": "UnityEngine.Color",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Color",
          "toString": "RGBA(0.000, 0.000, 0.000, 0.000)"
        }
      }
    },
    "489": {
      "address": 489,
      "type": "System.Object[]",
      "value": {
        "isSerializable": true,
        "value": [
          null,
          null,
          null,
          null,
          null
        ]
      }
    },
    "490": {
      "address": 490,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 28288
      }
    },
    "491": {
      "address": 491,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Udon"
      }
    },
    "492": {
      "address": 492,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen_Pen"
      }
    },
    "493": {
      "address": 493,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "</color>"
      }
    },
    "494": {
      "address": 494,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "[{0}{1}.{2}.{3}{4}] "
      }
    },
    "495": {
      "address": 495,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 360
      }
    },
    "496": {
      "address": 496,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 24
      }
    },
    "497": {
      "address": 497,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 255
      }
    },
    "498": {
      "address": 498,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 12
      }
    },
    "499": {
      "address": 499,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 4095
      }
    },
    "500": {
      "address": 500,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 8320864560273335438
      }
    },
    "501": {
      "address": 501,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__refl_typeid"
      }
    },
    "502": {
      "address": 502,
      "type": "System.UInt32[]",
      "value": {
        "isSerializable": true,
        "value": [
          30096,
          30136,
          30176,
          30216
        ]
      }
    },
    "503": {
      "address": 503,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "PenUsing"
      }
    },
    "504": {
      "address": 504,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "EraserUsing"
      }
    },
    "505": {
      "address": 505,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "(QvPen_Pen_State.???)"
      }
    },
    "506": {
      "address": 506,
      "type": "System.UInt32[]",
      "value": {
        "isSerializable": true,
        "value": [
          30612,
          30572,
          30452,
          30492,
          30532
        ]
      }
    },
    "507": {
      "address": 507,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 5
      }
    },
    "508": {
      "address": 508,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 31148
      }
    },
    "509": {
      "address": 509,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "510": {
      "address": 510,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 32872
      }
    },
    "511": {
      "address": 511,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 255.0
      }
    },
    "512": {
      "address": 512,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "{0:x2}{1:x2}{2:x2}"
      }
    },
    "513": {
      "address": 513,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "514": {
      "address": 514,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "515": {
      "address": 515,
      "type": "UnityEngine.SphereCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "516": {
      "address": 516,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "517": {
      "address": 517,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "518": {
      "address": 518,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "519": {
      "address": 519,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "520": {
      "address": 520,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "521": {
      "address": 521,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "522": {
      "address": 522,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "523": {
      "address": 523,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "524": {
      "address": 524,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "525": {
      "address": 525,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "526": {
      "address": 526,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "527": {
      "address": 527,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "528": {
      "address": 528,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "529": {
      "address": 529,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "530": {
      "address": 530,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "531": {
      "address": 531,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "532": {
      "address": 532,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "533": {
      "address": 533,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "534": {
      "address": 534,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "535": {
      "address": 535,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "536": {
      "address": 536,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "537": {
      "address": 537,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "538": {
      "address": 538,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "539": {
      "address": 539,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "540": {
      "address": 540,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "541": {
      "address": 541,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "542": {
      "address": 542,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "543": {
      "address": 543,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "544": {
      "address": 544,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "545": {
      "address": 545,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "546": {
      "address": 546,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "547": {
      "address": 547,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "548": {
      "address": 548,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "549": {
      "address": 549,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "550": {
      "address": 550,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "551": {
      "address": 551,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "552": {
      "address": 552,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "553": {
      "address": 553,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "554": {
      "address": 554,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "555": {
      "address": 555,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "556": {
      "address": 556,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "557": {
      "address": 557,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "558": {
      "address": 558,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "559": {
      "address": 559,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "560": {
      "address": 560,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "561": {
      "address": 561,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "562": {
      "address": 562,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "563": {
      "address": 563,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "564": {
      "address": 564,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "565": {
      "address": 565,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "566": {
      "address": 566,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "567": {
      "address": 567,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "568": {
      "address": 568,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "569": {
      "address": 569,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "570": {
      "address": 570,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "571": {
      "address": 571,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "572": {
      "address": 572,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "573": {
      "address": 573,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "574": {
      "address": 574,
      "type": "UnityEngine.Material",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "575": {
      "address": 575,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "576": {
      "address": 576,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "577": {
      "address": 577,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "578": {
      "address": 578,
      "type": "UnityEngine.Gradient",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "579": {
      "address": 579,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "580": {
      "address": 580,
      "type": "UnityEngine.Gradient",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "581": {
      "address": 581,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "582": {
      "address": 582,
      "type": "UnityEngine.LayerMask",
      "value": {
        "isSerializable": true,
        "value": {
          "value": 0
        }
      }
    },
    "583": {
      "address": 583,
      "type": "UnityEngine.Shader",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "584": {
      "address": 584,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "585": {
      "address": 585,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "586": {
      "address": 586,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "587": {
      "address": 587,
      "type": "UnityEngine.Shader",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "588": {
      "address": 588,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "589": {
      "address": 589,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "590": {
      "address": 590,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "591": {
      "address": 591,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "592": {
      "address": 592,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "593": {
      "address": 593,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "594": {
      "address": 594,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "595": {
      "address": 595,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "596": {
      "address": 596,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "597": {
      "address": 597,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "598": {
      "address": 598,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "599": {
      "address": 599,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "600": {
      "address": 600,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "601": {
      "address": 601,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "602": {
      "address": 602,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "603": {
      "address": 603,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "604": {
      "address": 604,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "605": {
      "address": 605,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "606": {
      "address": 606,
      "type": "UnityEngine.RectTransform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "607": {
      "address": 607,
      "type": "UnityEngine.Rect",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Rect",
          "toString": "(x:0.00, y:0.00, width:0.00, height:0.00)"
        }
      }
    },
    "608": {
      "address": 608,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "609": {
      "address": 609,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "610": {
      "address": 610,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "611": {
      "address": 611,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "612": {
      "address": 612,
      "type": "UnityEngine.Quaternion",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Quaternion",
          "toString": "(0.00000, 0.00000, 0.00000, 0.00000)"
        }
      }
    },
    "613": {
      "address": 613,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "614": {
      "address": 614,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "615": {
      "address": 615,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "616": {
      "address": 616,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "617": {
      "address": 617,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "618": {
      "address": 618,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "619": {
      "address": 619,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "620": {
      "address": 620,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "621": {
      "address": 621,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "622": {
      "address": 622,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "623": {
      "address": 623,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "624": {
      "address": 624,
      "type": "UnityEngine.Vector2",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector2",
          "toString": "(0.00, 0.00)"
        }
      }
    },
    "625": {
      "address": 625,
      "type": "UnityEngine.Vector2",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector2",
          "toString": "(0.00, 0.00)"
        }
      }
    },
    "626": {
      "address": 626,
      "type": "UnityEngine.Vector2",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector2",
          "toString": "(0.00, 0.00)"
        }
      }
    },
    "627": {
      "address": 627,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "628": {
      "address": 628,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "629": {
      "address": 629,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "630": {
      "address": 630,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "631": {
      "address": 631,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "632": {
      "address": 632,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "633": {
      "address": 633,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "634": {
      "address": 634,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "635": {
      "address": 635,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "636": {
      "address": 636,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "637": {
      "address": 637,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "638": {
      "address": 638,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "639": {
      "address": 639,
      "type": "UnityEngine.Quaternion",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Quaternion",
          "toString": "(0.00000, 0.00000, 0.00000, 0.00000)"
        }
      }
    },
    "640": {
      "address": 640,
      "type": "UnityEngine.Quaternion",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Quaternion",
          "toString": "(0.00000, 0.00000, 0.00000, 0.00000)"
        }
      }
    },
    "641": {
      "address": 641,
      "type": "UnityEngine.Quaternion",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Quaternion",
          "toString": "(0.00000, 0.00000, 0.00000, 0.00000)"
        }
      }
    },
    "642": {
      "address": 642,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "643": {
      "address": 643,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "644": {
      "address": 644,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "645": {
      "address": 645,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "646": {
      "address": 646,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "647": {
      "address": 647,
      "type": "UnityEngine.Collider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "648": {
      "address": 648,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "649": {
      "address": 649,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "650": {
      "address": 650,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "651": {
      "address": 651,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "652": {
      "address": 652,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "653": {
      "address": 653,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "654": {
      "address": 654,
      "type": "UnityEngine.Collider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "655": {
      "address": 655,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "656": {
      "address": 656,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "657": {
      "address": 657,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "658": {
      "address": 658,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "659": {
      "address": 659,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "660": {
      "address": 660,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "661": {
      "address": 661,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "662": {
      "address": 662,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "663": {
      "address": 663,
      "type": "UnityEngine.LineRenderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "664": {
      "address": 664,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "665": {
      "address": 665,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "666": {
      "address": 666,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "667": {
      "address": 667,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "668": {
      "address": 668,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "669": {
      "address": 669,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "670": {
      "address": 670,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "671": {
      "address": 671,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "672": {
      "address": 672,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "673": {
      "address": 673,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "674": {
      "address": 674,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "675": {
      "address": 675,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "676": {
      "address": 676,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "677": {
      "address": 677,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "678": {
      "address": 678,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "679": {
      "address": 679,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "680": {
      "address": 680,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "681": {
      "address": 681,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "682": {
      "address": 682,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "683": {
      "address": 683,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "684": {
      "address": 684,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "685": {
      "address": 685,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "686": {
      "address": 686,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "687": {
      "address": 687,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "688": {
      "address": 688,
      "type": "UnityEngine.Quaternion",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Quaternion",
          "toString": "(0.00000, 0.00000, 0.00000, 0.00000)"
        }
      }
    },
    "689": {
      "address": 689,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "690": {
      "address": 690,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "691": {
      "address": 691,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "692": {
      "address": 692,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "693": {
      "address": 693,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "694": {
      "address": 694,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "695": {
      "address": 695,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "696": {
      "address": 696,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "697": {
      "address": 697,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "698": {
      "address": 698,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "699": {
      "address": 699,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "700": {
      "address": 700,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "701": {
      "address": 701,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "702": {
      "address": 702,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "703": {
      "address": 703,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "704": {
      "address": 704,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "705": {
      "address": 705,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "706": {
      "address": 706,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "707": {
      "address": 707,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "708": {
      "address": 708,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "709": {
      "address": 709,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "710": {
      "address": 710,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "711": {
      "address": 711,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "712": {
      "address": 712,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "713": {
      "address": 713,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "714": {
      "address": 714,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "715": {
      "address": 715,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "716": {
      "address": 716,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "717": {
      "address": 717,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "718": {
      "address": 718,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "719": {
      "address": 719,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "720": {
      "address": 720,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "721": {
      "address": 721,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "722": {
      "address": 722,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "723": {
      "address": 723,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "724": {
      "address": 724,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "725": {
      "address": 725,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "726": {
      "address": 726,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "727": {
      "address": 727,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "728": {
      "address": 728,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "729": {
      "address": 729,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "730": {
      "address": 730,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "731": {
      "address": 731,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "732": {
      "address": 732,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "733": {
      "address": 733,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "734": {
      "address": 734,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "735": {
      "address": 735,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "736": {
      "address": 736,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "737": {
      "address": 737,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "738": {
      "address": 738,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "739": {
      "address": 739,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "740": {
      "address": 740,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "741": {
      "address": 741,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "742": {
      "address": 742,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "743": {
      "address": 743,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "744": {
      "address": 744,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "745": {
      "address": 745,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "746": {
      "address": 746,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "747": {
      "address": 747,
      "type": "UnityEngine.TrailRenderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "748": {
      "address": 748,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "749": {
      "address": 749,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "750": {
      "address": 750,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "751": {
      "address": 751,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "752": {
      "address": 752,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "753": {
      "address": 753,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "754": {
      "address": 754,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "755": {
      "address": 755,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "756": {
      "address": 756,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "757": {
      "address": 757,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "758": {
      "address": 758,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "759": {
      "address": 759,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "760": {
      "address": 760,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "761": {
      "address": 761,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "762": {
      "address": 762,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "763": {
      "address": 763,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "764": {
      "address": 764,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "765": {
      "address": 765,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "766": {
      "address": 766,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "767": {
      "address": 767,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "768": {
      "address": 768,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "769": {
      "address": 769,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "770": {
      "address": 770,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "771": {
      "address": 771,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "772": {
      "address": 772,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "773": {
      "address": 773,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "774": {
      "address": 774,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "775": {
      "address": 775,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "776": {
      "address": 776,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "777": {
      "address": 777,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "778": {
      "address": 778,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "779": {
      "address": 779,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "780": {
      "address": 780,
      "type": "UnityEngine.Vector3Int",
      "value": {
        "isSerializable": true,
        "value": {
          "x": 0,
          "y": 0,
          "z": 0,
          "magnitude": 0.0,
          "sqrMagnitude": 0
        }
      }
    },
    "781": {
      "address": 781,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "782": {
      "address": 782,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "783": {
      "address": 783,
      "type": "UnityEngine.Vector3Int",
      "value": {
        "isSerializable": true,
        "value": {
          "x": 0,
          "y": 0,
          "z": 0,
          "magnitude": 0.0,
          "sqrMagnitude": 0
        }
      }
    },
    "784": {
      "address": 784,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "785": {
      "address": 785,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "786": {
      "address": 786,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "787": {
      "address": 787,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "788": {
      "address": 788,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "789": {
      "address": 789,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "790": {
      "address": 790,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "791": {
      "address": 791,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "792": {
      "address": 792,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "793": {
      "address": 793,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "794": {
      "address": 794,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "795": {
      "address": 795,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "796": {
      "address": 796,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "797": {
      "address": 797,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "798": {
      "address": 798,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "799": {
      "address": 799,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "800": {
      "address": 800,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "801": {
      "address": 801,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "802": {
      "address": 802,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "803": {
      "address": 803,
      "type": "VRC.SDK3.Data.DataToken",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "VRC.SDK3.Data.DataToken",
          "toString": "Null"
        }
      }
    },
    "804": {
      "address": 804,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "805": {
      "address": 805,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "806": {
      "address": 806,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "807": {
      "address": 807,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "808": {
      "address": 808,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "809": {
      "address": 809,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "810": {
      "address": 810,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "811": {
      "address": 811,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "812": {
      "address": 812,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "813": {
      "address": 813,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "814": {
      "address": 814,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "815": {
      "address": 815,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "816": {
      "address": 816,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "817": {
      "address": 817,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "818": {
      "address": 818,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "819": {
      "address": 819,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "820": {
      "address": 820,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "821": {
      "address": 821,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "822": {
      "address": 822,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "823": {
      "address": 823,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "824": {
      "address": 824,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "825": {
      "address": 825,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "826": {
      "address": 826,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "827": {
      "address": 827,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "828": {
      "address": 828,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "829": {
      "address": 829,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "830": {
      "address": 830,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "831": {
      "address": 831,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "832": {
      "address": 832,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "833": {
      "address": 833,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "834": {
      "address": 834,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "835": {
      "address": 835,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "836": {
      "address": 836,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "837": {
      "address": 837,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "838": {
      "address": 838,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "839": {
      "address": 839,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "840": {
      "address": 840,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "841": {
      "address": 841,
      "type": "UnityEngine.LineRenderer",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "842": {
      "address": 842,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "843": {
      "address": 843,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "844": {
      "address": 844,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "845": {
      "address": 845,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "846": {
      "address": 846,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "847": {
      "address": 847,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "848": {
      "address": 848,
      "type": "UnityEngine.Mesh",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "849": {
      "address": 849,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "850": {
      "address": 850,
      "type": "UnityEngine.MeshCollider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "851": {
      "address": 851,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "852": {
      "address": 852,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "853": {
      "address": 853,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "854": {
      "address": 854,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "855": {
      "address": 855,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "856": {
      "address": 856,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "857": {
      "address": 857,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "858": {
      "address": 858,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "859": {
      "address": 859,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "860": {
      "address": 860,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "861": {
      "address": 861,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "862": {
      "address": 862,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "863": {
      "address": 863,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "864": {
      "address": 864,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "865": {
      "address": 865,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "866": {
      "address": 866,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "867": {
      "address": 867,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "868": {
      "address": 868,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "869": {
      "address": 869,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "870": {
      "address": 870,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "871": {
      "address": 871,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "872": {
      "address": 872,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "873": {
      "address": 873,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "874": {
      "address": 874,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "875": {
      "address": 875,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "876": {
      "address": 876,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "877": {
      "address": 877,
      "type": "UnityEngine.Vector3[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "878": {
      "address": 878,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "879": {
      "address": 879,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "880": {
      "address": 880,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "881": {
      "address": 881,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "882": {
      "address": 882,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "883": {
      "address": 883,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "884": {
      "address": 884,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "885": {
      "address": 885,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "886": {
      "address": 886,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "887": {
      "address": 887,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "888": {
      "address": 888,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "889": {
      "address": 889,
      "type": "UnityEngine.Collider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "890": {
      "address": 890,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "891": {
      "address": 891,
      "type": "UnityEngine.Collider",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "892": {
      "address": 892,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "893": {
      "address": 893,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "894": {
      "address": 894,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "895": {
      "address": 895,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "896": {
      "address": 896,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "897": {
      "address": 897,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "898": {
      "address": 898,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "899": {
      "address": 899,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "900": {
      "address": 900,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "901": {
      "address": 901,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "902": {
      "address": 902,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "903": {
      "address": 903,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "904": {
      "address": 904,
      "type": "UnityEngine.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "905": {
      "address": 905,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "906": {
      "address": 906,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "907": {
      "address": 907,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "908": {
      "address": 908,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "909": {
      "address": 909,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "910": {
      "address": 910,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "911": {
      "address": 911,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "912": {
      "address": 912,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "913": {
      "address": 913,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "914": {
      "address": 914,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "915": {
      "address": 915,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "916": {
      "address": 916,
      "type": "UnityEngine.Vector3",
      "value": {
        "isSerializable": false,
        "value": {
          "type": "UnityEngine.Vector3",
          "toString": "(0.00, 0.00, 0.00)"
        }
      }
    },
    "917": {
      "address": 917,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "918": {
      "address": 918,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "919": {
      "address": 919,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "920": {
      "address": 920,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "921": {
      "address": 921,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "922": {
      "address": 922,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "923": {
      "address": 923,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "924": {
      "address": 924,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "925": {
      "address": 925,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "926": {
      "address": 926,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "927": {
      "address": 927,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "928": {
      "address": 928,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "929": {
      "address": 929,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "930": {
      "address": 930,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "931": {
      "address": 931,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "932": {
      "address": 932,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "933": {
      "address": 933,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "934": {
      "address": 934,
      "type": "System.Type",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "935": {
      "address": 935,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "936": {
      "address": 936,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "937": {
      "address": 937,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "938": {
      "address": 938,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "939": {
      "address": 939,
      "type": "UnityEngine.Component",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "940": {
      "address": 940,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "941": {
      "address": 941,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "942": {
      "address": 942,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "943": {
      "address": 943,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "944": {
      "address": 944,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "945": {
      "address": 945,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "946": {
      "address": 946,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "947": {
      "address": 947,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "948": {
      "address": 948,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "949": {
      "address": 949,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "950": {
      "address": 950,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "951": {
      "address": 951,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "952": {
      "address": 952,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "953": {
      "address": 953,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "954": {
      "address": 954,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "955": {
      "address": 955,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "956": {
      "address": 956,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "957": {
      "address": 957,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "958": {
      "address": 958,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "959": {
      "address": 959,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "960": {
      "address": 960,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "961": {
      "address": 961,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "962": {
      "address": 962,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "963": {
      "address": 963,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "964": {
      "address": 964,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "965": {
      "address": 965,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "966": {
      "address": 966,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "967": {
      "address": 967,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "968": {
      "address": 968,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "969": {
      "address": 969,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "970": {
      "address": 970,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "971": {
      "address": 971,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "972": {
      "address": 972,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "973": {
      "address": 973,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "974": {
      "address": 974,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "975": {
      "address": 975,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "976": {
      "address": 976,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "977": {
      "address": 977,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "978": {
      "address": 978,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "979": {
      "address": 979,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "980": {
      "address": 980,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "981": {
      "address": 981,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "982": {
      "address": 982,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "983": {
      "address": 983,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "984": {
      "address": 984,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "985": {
      "address": 985,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "986": {
      "address": 986,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "987": {
      "address": 987,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "988": {
      "address": 988,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "989": {
      "address": 989,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "990": {
      "address": 990,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "991": {
      "address": 991,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "992": {
      "address": 992,
      "type": "System.Double",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "993": {
      "address": 993,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "994": {
      "address": 994,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "995": {
      "address": 995,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "996": {
      "address": 996,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "997": {
      "address": 997,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "998": {
      "address": 998,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "999": {
      "address": 999,
      "type": "UnityEngine.Transform",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "1000": {
      "address": 1000,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "1001": {
      "address": 1001,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "1002": {
      "address": 1002,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "1003": {
      "address": 1003,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "1004": {
      "address": 1004,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "1005": {
      "address": 1005,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "1006": {
      "address": 1006,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "1007": {
      "address": 1007,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__get_transform__UnityEngineTransform"
      }
    },
    "1008": {
      "address": 1008,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__GetComponent__T"
      }
    },
    "1009": {
      "address": 1009,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineCollider.__set_enabled__SystemBoolean__SystemVoid"
      }
    },
    "1010": {
      "address": 1010,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_lossyScale__UnityEngineVector3"
      }
    },
    "1011": {
      "address": 1011,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__get_x__SystemSingle"
      }
    },
    "1012": {
      "address": 1012,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingleArray.__Set__SystemInt32_SystemSingle__SystemVoid"
      }
    },
    "1013": {
      "address": 1013,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__get_y__SystemSingle"
      }
    },
    "1014": {
      "address": 1014,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__get_z__SystemSingle"
      }
    },
    "1015": {
      "address": 1015,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__Min__SystemSingleArray__SystemSingle"
      }
    },
    "1016": {
      "address": 1016,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__Max__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "1017": {
      "address": 1017,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineSphereCollider.__get_radius__SystemSingle"
      }
    },
    "1018": {
      "address": 1018,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Multiplication__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "1019": {
      "address": 1019,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__Abs__SystemSingle__SystemSingle"
      }
    },
    "1020": {
      "address": 1020,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__IsValid__SystemObject__SystemBoolean"
      }
    },
    "1021": {
      "address": 1021,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__GetComponentInChildren__SystemBoolean__T"
      }
    },
    "1022": {
      "address": 1022,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__GetComponent__SystemType__UnityEngineComponent"
      }
    },
    "1023": {
      "address": 1023,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObject.__op_Equality__SystemObject_SystemObject__SystemBoolean"
      }
    },
    "1024": {
      "address": 1024,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__get_LocalPlayer__VRCSDKBaseVRCPlayerApi"
      }
    },
    "1025": {
      "address": 1025,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__get_playerId__SystemInt32"
      }
    },
    "1026": {
      "address": 1026,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__IsUserInVR__SystemBoolean"
      }
    },
    "1027": {
      "address": 1027,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject__SystemString"
      }
    },
    "1028": {
      "address": 1028,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__Find__SystemString__UnityEngineGameObject"
      }
    },
    "1029": {
      "address": 1029,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__get_gameObject__UnityEngineGameObject"
      }
    },
    "1030": {
      "address": 1030,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__SetActive__SystemBoolean__SystemVoid"
      }
    },
    "1031": {
      "address": 1031,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__get_transform__UnityEngineTransform"
      }
    },
    "1032": {
      "address": 1032,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__set_name__SystemString__SystemVoid"
      }
    },
    "1033": {
      "address": 1033,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__SetAsFirstSibling__SystemVoid"
      }
    },
    "1034": {
      "address": 1034,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__GetUniqueName__UnityEngineGameObject__SystemString"
      }
    },
    "1035": {
      "address": 1035,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__IsNullOrEmpty__SystemString__SystemBoolean"
      }
    },
    "1036": {
      "address": 1036,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__GetHashCode__SystemInt32"
      }
    },
    "1037": {
      "address": 1037,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemConvert.__ToDouble__SystemSingle__SystemDouble"
      }
    },
    "1038": {
      "address": 1038,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemMath.__Truncate__SystemDouble__SystemDouble"
      }
    },
    "1039": {
      "address": 1039,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemConvert.__ToInt32__SystemDouble__SystemInt32"
      }
    },
    "1040": {
      "address": 1040,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject_SystemObject_SystemObject__SystemString"
      }
    },
    "1041": {
      "address": 1041,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject_SystemObject__SystemString"
      }
    },
    "1042": {
      "address": 1042,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEvent__SystemString__SystemVoid"
      }
    },
    "1043": {
      "address": 1043,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariable__SystemString__SystemObject"
      }
    },
    "1044": {
      "address": 1044,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SetProgramVariable__SystemString_SystemObject__SystemVoid"
      }
    },
    "1045": {
      "address": 1045,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3ComponentsVRCPickup.__set_InteractionText__SystemString"
      }
    },
    "1046": {
      "address": 1046,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3ComponentsVRCPickup.__set_UseText__SystemString"
      }
    },
    "1047": {
      "address": 1047,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_localScale__UnityEngineVector3"
      }
    },
    "1048": {
      "address": 1048,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Multiply__UnityEngineVector3_SystemSingle__UnityEngineVector3"
      }
    },
    "1049": {
      "address": 1049,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__set_localScale__UnityEngineVector3__SystemVoid"
      }
    },
    "1050": {
      "address": 1050,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemConvert.__ToSingle__SystemObject__SystemSingle"
      }
    },
    "1051": {
      "address": 1051,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemConvert.__ToInt32__SystemObject__SystemInt32"
      }
    },
    "1052": {
      "address": 1052,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LeftShift__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1053": {
      "address": 1053,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__set_layer__SystemInt32__SystemVoid"
      }
    },
    "1054": {
      "address": 1054,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineRenderer.__set_material__UnityEngineMaterial__SystemVoid"
      }
    },
    "1055": {
      "address": 1055,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMaterial.__get_shader__UnityEngineShader"
      }
    },
    "1056": {
      "address": 1056,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__op_Equality__UnityEngineObject_UnityEngineObject__SystemBoolean"
      }
    },
    "1057": {
      "address": 1057,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__get_name__SystemString"
      }
    },
    "1058": {
      "address": 1058,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Contains__SystemString__SystemBoolean"
      }
    },
    "1059": {
      "address": 1059,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_LogicalOr__SystemBoolean_SystemBoolean__SystemBoolean"
      }
    },
    "1060": {
      "address": 1060,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLineRenderer.__set_widthMultiplier__SystemSingle__SystemVoid"
      }
    },
    "1061": {
      "address": 1061,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMaterialPropertyBlock.__ctor____UnityEngineMaterialPropertyBlock"
      }
    },
    "1062": {
      "address": 1062,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineRenderer.__GetPropertyBlock__UnityEngineMaterialPropertyBlock__SystemVoid"
      }
    },
    "1063": {
      "address": 1063,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMaterialPropertyBlock.__SetFloat__SystemString_SystemSingle__SystemVoid"
      }
    },
    "1064": {
      "address": 1064,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineRenderer.__SetPropertyBlock__UnityEngineMaterialPropertyBlock__SystemVoid"
      }
    },
    "1065": {
      "address": 1065,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTrailRenderer.__set_widthMultiplier__SystemSingle__SystemVoid"
      }
    },
    "1066": {
      "address": 1066,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMaterialPropertyBlock.__Clear__SystemVoid"
      }
    },
    "1067": {
      "address": 1067,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLineRenderer.__set_colorGradient__UnityEngineGradient__SystemVoid"
      }
    },
    "1068": {
      "address": 1068,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTrailRenderer.__set_colorGradient__UnityEngineGradient__SystemVoid"
      }
    },
    "1069": {
      "address": 1069,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLayerMask.__op_Implicit__UnityEngineLayerMask__SystemInt32"
      }
    },
    "1070": {
      "address": 1070,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Equality__UnityEngineVector3_UnityEngineVector3__SystemBoolean"
      }
    },
    "1071": {
      "address": 1071,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemBoolean.__op_UnaryNegation__SystemBoolean__SystemBoolean"
      }
    },
    "1072": {
      "address": 1072,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineInput.__GetKeyUp__UnityEngineKeyCode__SystemBoolean"
      }
    },
    "1073": {
      "address": 1073,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineInput.__get_anyKey__SystemBoolean"
      }
    },
    "1074": {
      "address": 1074,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineInput.__GetKeyDown__UnityEngineKeyCode__SystemBoolean"
      }
    },
    "1075": {
      "address": 1075,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineInput.__GetKey__UnityEngineKeyCode__SystemBoolean"
      }
    },
    "1076": {
      "address": 1076,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomNetworkEvent__VRCUdonCommonInterfacesNetworkEventTarget_SystemString__SystemVoid"
      }
    },
    "1077": {
      "address": 1077,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Addition__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "1078": {
      "address": 1078,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__Min__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "1079": {
      "address": 1079,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Subtraction__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "1080": {
      "address": 1080,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineRenderer.__set_enabled__SystemBoolean__SystemVoid"
      }
    },
    "1081": {
      "address": 1081,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineRectTransform.__get_rect__UnityEngineRect"
      }
    },
    "1082": {
      "address": 1082,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineRect.__get_size__UnityEngineVector2"
      }
    },
    "1083": {
      "address": 1083,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__op_Division__UnityEngineVector2_SystemSingle__UnityEngineVector2"
      }
    },
    "1084": {
      "address": 1084,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__get_y__SystemSingle"
      }
    },
    "1085": {
      "address": 1085,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Division__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "1086": {
      "address": 1086,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__SetLocalPositionAndRotation__UnityEngineVector3_UnityEngineQuaternion__SystemVoid"
      }
    },
    "1087": {
      "address": 1087,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_position__UnityEngineVector3"
      }
    },
    "1088": {
      "address": 1088,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_rotation__UnityEngineQuaternion"
      }
    },
    "1089": {
      "address": 1089,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__SetPositionAndRotation__UnityEngineVector3_UnityEngineQuaternion__SystemVoid"
      }
    },
    "1090": {
      "address": 1090,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__GetTrackingData__VRCSDKBaseVRCPlayerApiTrackingDataType__VRCSDKBaseVRCPlayerApiTrackingData"
      }
    },
    "1091": {
      "address": 1091,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApiTrackingData.__get_position__UnityEngineVector3"
      }
    },
    "1092": {
      "address": 1092,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApiTrackingData.__get_rotation__UnityEngineQuaternion"
      }
    },
    "1093": {
      "address": 1093,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineQuaternion.__op_Multiply__UnityEngineQuaternion_UnityEngineVector3__UnityEngineVector3"
      }
    },
    "1094": {
      "address": 1094,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Subtraction__UnityEngineVector3_UnityEngineVector3__UnityEngineVector3"
      }
    },
    "1095": {
      "address": 1095,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__Dot__UnityEngineVector3_UnityEngineVector3__SystemSingle"
      }
    },
    "1096": {
      "address": 1096,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Addition__UnityEngineVector3_UnityEngineVector3__UnityEngineVector3"
      }
    },
    "1097": {
      "address": 1097,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineInput.__GetAxis__SystemString__SystemSingle"
      }
    },
    "1098": {
      "address": 1098,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__set_x__SystemSingle"
      }
    },
    "1099": {
      "address": 1099,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__set_y__SystemSingle"
      }
    },
    "1100": {
      "address": 1100,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTime.__get_deltaTime__SystemSingle"
      }
    },
    "1101": {
      "address": 1101,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__op_Multiply__SystemSingle_UnityEngineVector2__UnityEngineVector2"
      }
    },
    "1102": {
      "address": 1102,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__op_Addition__UnityEngineVector2_UnityEngineVector2__UnityEngineVector2"
      }
    },
    "1103": {
      "address": 1103,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__op_UnaryNegation__UnityEngineVector2__UnityEngineVector2"
      }
    },
    "1104": {
      "address": 1104,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__Max__UnityEngineVector2_UnityEngineVector2__UnityEngineVector2"
      }
    },
    "1105": {
      "address": 1105,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__Min__UnityEngineVector2_UnityEngineVector2__UnityEngineVector2"
      }
    },
    "1106": {
      "address": 1106,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector2.__op_Implicit__UnityEngineVector2__UnityEngineVector3"
      }
    },
    "1107": {
      "address": 1107,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineCollider.__ClosestPoint__UnityEngineVector3__UnityEngineVector3"
      }
    },
    "1108": {
      "address": 1108,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__Distance__UnityEngineVector3_UnityEngineVector3__SystemSingle"
      }
    },
    "1109": {
      "address": 1109,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__MoveTowards__UnityEngineVector3_UnityEngineVector3_SystemSingle__UnityEngineVector3"
      }
    },
    "1110": {
      "address": 1110,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__set_position__UnityEngineVector3__SystemVoid"
      }
    },
    "1111": {
      "address": 1111,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_GreaterThan__SystemSingle_SystemSingle__SystemBoolean"
      }
    },
    "1112": {
      "address": 1112,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__Lerp__UnityEngineVector3_UnityEngineVector3_SystemSingle__UnityEngineVector3"
      }
    },
    "1113": {
      "address": 1113,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineQuaternion.__Lerp__UnityEngineQuaternion_UnityEngineQuaternion_SystemSingle__UnityEngineQuaternion"
      }
    },
    "1114": {
      "address": 1114,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEnginePhysics.__OverlapSphereNonAlloc__UnityEngineVector3_SystemSingle_UnityEngineColliderArray_SystemInt32_UnityEngineQueryTriggerInteraction__SystemInt32"
      }
    },
    "1115": {
      "address": 1115,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "1116": {
      "address": 1116,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Get__SystemInt32__SystemObject"
      }
    },
    "1117": {
      "address": 1117,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_parent__UnityEngineTransform"
      }
    },
    "1118": {
      "address": 1118,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__GetComponentInParent__T"
      }
    },
    "1119": {
      "address": 1119,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLineRenderer.__get_positionCount__SystemInt32"
      }
    },
    "1120": {
      "address": 1120,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "1121": {
      "address": 1121,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Set__SystemInt32_SystemObject__SystemVoid"
      }
    },
    "1122": {
      "address": 1122,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1123": {
      "address": 1123,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__op_Implicit__UnityEngineObject__SystemBoolean"
      }
    },
    "1124": {
      "address": 1124,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineCollider.__get_isTrigger__SystemBoolean"
      }
    },
    "1125": {
      "address": 1125,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__get_layer__SystemInt32"
      }
    },
    "1126": {
      "address": 1126,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LogicalAnd__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1127": {
      "address": 1127,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "1128": {
      "address": 1128,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObject.__GetType__SystemType"
      }
    },
    "1129": {
      "address": 1129,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemType.__op_Equality__SystemType_SystemType__SystemBoolean"
      }
    },
    "1130": {
      "address": 1130,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMeshCollider.__get_convex__SystemBoolean"
      }
    },
    "1131": {
      "address": 1131,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_LessThan__SystemSingle_SystemSingle__SystemBoolean"
      }
    },
    "1132": {
      "address": 1132,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTime.__get_time__SystemSingle"
      }
    },
    "1133": {
      "address": 1133,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_GreaterThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "1134": {
      "address": 1134,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThanOrEqual__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "1135": {
      "address": 1135,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemUInt32Array.__Get__SystemInt32__SystemUInt32"
      }
    },
    "1136": {
      "address": 1136,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__Destroy__UnityEngineObject__SystemVoid"
      }
    },
    "1137": {
      "address": 1137,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__IsOwner__UnityEngineGameObject__SystemBoolean"
      }
    },
    "1138": {
      "address": 1138,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__SetOwner__VRCSDKBaseVRCPlayerApi_UnityEngineGameObject__SystemVoid"
      }
    },
    "1139": {
      "address": 1139,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3ComponentsVRCPickup.__Drop__SystemVoid"
      }
    },
    "1140": {
      "address": 1140,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3ComponentsVRCObjectSync.__Respawn__SystemVoid"
      }
    },
    "1141": {
      "address": 1141,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTrailRenderer.__Clear__SystemVoid"
      }
    },
    "1142": {
      "address": 1142,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTrailRenderer.__get_positionCount__SystemInt32"
      }
    },
    "1143": {
      "address": 1143,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3Array.__ctor__SystemInt32__UnityEngineVector3Array"
      }
    },
    "1144": {
      "address": 1144,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTrailRenderer.__GetPositions__UnityEngineVector3Array__SystemInt32"
      }
    },
    "1145": {
      "address": 1145,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__Reverse__SystemArray__SystemVoid"
      }
    },
    "1146": {
      "address": 1146,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__Copy__SystemArray_SystemArray_SystemInt32__SystemVoid"
      }
    },
    "1147": {
      "address": 1147,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemConvert.__ToSingle__SystemInt32__SystemSingle"
      }
    },
    "1148": {
      "address": 1148,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__ctor__SystemSingle_SystemSingle_SystemSingle__UnityEngineVector3"
      }
    },
    "1149": {
      "address": 1149,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLineRenderer.__GetPositions__UnityEngineVector3Array__SystemInt32"
      }
    },
    "1150": {
      "address": 1150,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3Int.__ctor__SystemInt32_SystemInt32_SystemInt32__UnityEngineVector3Int"
      }
    },
    "1151": {
      "address": 1151,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3Int.__op_Implicit__UnityEngineVector3Int__UnityEngineVector3"
      }
    },
    "1152": {
      "address": 1152,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineRenderer.__set_sharedMaterial__UnityEngineMaterial__SystemVoid"
      }
    },
    "1153": {
      "address": 1153,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Inequality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "1154": {
      "address": 1154,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__get_Count__SystemInt32"
      }
    },
    "1155": {
      "address": 1155,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__RemoveAt__SystemInt32__SystemVoid"
      }
    },
    "1156": {
      "address": 1156,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__op_Implicit__SystemInt32__VRCSDK3DataDataToken"
      }
    },
    "1157": {
      "address": 1157,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__Add__VRCSDK3DataDataToken__SystemVoid"
      }
    },
    "1158": {
      "address": 1158,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Subtraction__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1159": {
      "address": 1159,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataList.__TryGetValue__SystemInt32_VRCSDK3DataTokenType_VRCSDK3DataDataTokenRef__SystemBoolean"
      }
    },
    "1160": {
      "address": 1160,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDK3DataDataToken.__get_Int__SystemInt32"
      }
    },
    "1161": {
      "address": 1161,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
      }
    },
    "1162": {
      "address": 1162,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLineRenderer.__set_positionCount__SystemInt32__SystemVoid"
      }
    },
    "1163": {
      "address": 1163,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLineRenderer.__SetPositions__UnityEngineVector3Array__SystemVoid"
      }
    },
    "1164": {
      "address": 1164,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMesh.__ctor____UnityEngineMesh"
      }
    },
    "1165": {
      "address": 1165,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLineRenderer.__get_widthMultiplier__SystemSingle"
      }
    },
    "1166": {
      "address": 1166,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineLineRenderer.__BakeMesh__UnityEngineMesh_SystemBoolean__SystemVoid"
      }
    },
    "1167": {
      "address": 1167,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMeshCollider.__set_sharedMesh__UnityEngineMesh__SystemVoid"
      }
    },
    "1168": {
      "address": 1168,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__GetComponents__SystemType__UnityEngineComponentArray"
      }
    },
    "1169": {
      "address": 1169,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__get_DisableInteractive__SystemBoolean"
      }
    },
    "1170": {
      "address": 1170,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__Clear__SystemArray_SystemInt32_SystemInt32__SystemVoid"
      }
    },
    "1171": {
      "address": 1171,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__Log__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "1172": {
      "address": 1172,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__LogWarning__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "1173": {
      "address": 1173,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineDebug.__LogError__SystemObject_UnityEngineObject__SystemVoid"
      }
    },
    "1174": {
      "address": 1174,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObjectArray__SystemString"
      }
    },
    "1175": {
      "address": 1175,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Division__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1176": {
      "address": 1176,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Remainder__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1177": {
      "address": 1177,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3.__op_Division__UnityEngineVector3_SystemSingle__UnityEngineVector3"
      }
    },
    "1178": {
      "address": 1178,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__SetParent__UnityEngineTransform__SystemVoid"
      }
    },
    "1179": {
      "address": 1179,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_RightShift__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1180": {
      "address": 1180,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariableType__SystemString__SystemType"
      }
    },
    "1181": {
      "address": 1181,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObject.__op_Inequality__SystemObject_SystemObject__SystemBoolean"
      }
    },
    "1182": {
      "address": 1182,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemConvert.__ToInt64__SystemObject__SystemInt64"
      }
    },
    "1183": {
      "address": 1183,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt64.__op_Equality__SystemInt64_SystemInt64__SystemBoolean"
      }
    },
    "1184": {
      "address": 1184,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3Array.__Set__SystemInt32_UnityEngineVector3__SystemVoid"
      }
    },
    "1185": {
      "address": 1185,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineVector3Array.__Get__SystemInt32__UnityEngineVector3"
      }
    },
    "1186": {
      "address": 1186,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LogicalOr__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1187": {
      "address": 1187,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCInstantiate.__Instantiate__UnityEngineGameObject__UnityEngineGameObject"
      }
    },
    "1188": {
      "address": 1188,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_childCount__SystemInt32"
      }
    },
    "1189": {
      "address": 1189,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__GetChild__SystemInt32__UnityEngineTransform"
      }
    },
    "1190": {
      "address": 1190,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__set_localPosition__UnityEngineVector3__SystemVoid"
      }
    },
    "1191": {
      "address": 1191,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__set_localEulerAngles__UnityEngineVector3__SystemVoid"
      }
    },
    "1192": {
      "address": 1192,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__Clamp__SystemInt32_SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "1193": {
      "address": 1193,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_localPosition__UnityEngineVector3"
      }
    },
    "1194": {
      "address": 1194,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTransform.__get_localEulerAngles__UnityEngineVector3"
      }
    },
    "1195": {
      "address": 1195,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__op_Multiply__UnityEngineColor_SystemSingle__UnityEngineColor"
      }
    },
    "1196": {
      "address": 1196,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_r__SystemSingle"
      }
    },
    "1197": {
      "address": 1197,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__RoundToInt__SystemSingle__SystemInt32"
      }
    },
    "1198": {
      "address": 1198,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_g__SystemSingle"
      }
    },
    "1199": {
      "address": 1199,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineColor.__get_b__SystemSingle"
      }
    }
  }
}
```