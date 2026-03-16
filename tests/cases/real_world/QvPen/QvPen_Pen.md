<!-- ci: skip-compile -->

```csharp
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

```hex
1f8b08000000000002ffec7d077c14c5fbfeed6d08bd295dd0d00352d2e89d84269d84223d9000818484247429f6868a8a8a1d15153b76ecbd63efbd2bf6fab597ffceec4ceeddc9bc773bb7bb975dff3ffde866f7769f79df67de69efccbc13ee1d32fed17a18ff9b119a16ca0ef50a4d0f15844a432b8dbfb28d6b89f1af79c79f4f09951bd7a5c6fff38ddf7a8452627c49fee9aab532fe3fceb8ab08551a5fae0e2d36ae45f4f7dc50a1f1f738e3db42e38e3c5d62fc5768bca525191f4d0fe584268726855a6acd6340cca01f55b0671ad14bab63fc7f6468bdf16b21158a24d29bfed4d2f85faef14b05fdadc4109abf3727348faa5562fcb6d8f8a63c546c602ea288759a1c1fa27f84cccb4476adcfae138cffeab2bfc3ecef3aec6ff87e63e33fa2dd3a76df845d2745f9be9671c962bf3532af6182911c0ae9df817bf2fb6476afb3dfbf07f7e4f729ecf90f025e2df6fc47704faec9ecf94fecfd69ec9a2bfcfeb3f05d6df6fc7fc27779c2efe27775d8f35f84efa60bbfff6cfd3d5c973dff953d9f0a9e936b3df6fb6f563ec2f5d9f3dfc17bfc39cf1ff2fb1fec7e06c8c7ba0a7918cb2e6699f95cfb22767fa46927e16dec7e36bbce6418c42676b3674dc16f44d63f6da4d78ae9d9807df317b837d2d593acef1965a32a5d7d167bc6de0937146cb1a1c0d55c8045defb1bc18e25736b769d67f3fd364c9e46828e8d988e07acef85e647744c4a63cf16b0eb42f66d6386f50fb8e7187511bc58721ec2b09a08723631e54cda6e7dcfa87e2372bec19e2d62d7c5ecdba6829c4d01465d042f969c87b36b81033b2f045876d2e4f9b004bcbf845d9732dd0e62bafe0bee0dde6abd0030924d734531a3c9906ee225737bcd60d765917c48dec39e1599f7b59609b2a65bf3b60a7307bb5f8e7c97c164d76ca49d2bfc164baf4c26039789b7312b627c5fccaea54ce6af41bae4ba32225b297f5602d2a82bd40f99e0995d19a2e9d5c7d4abf693ecbe2fa853995c750ad8b33284f73e427ef561b67570442efab7914e9d29ec9d5508565f968761ab3cd5b0a08cb1742c67d743c1fb1566fa4db99d57b2eb6a96467326870eeec9b5197b9e04eec9b585501fb530f5adcbdaf0506df69cdb682d700ffa1a49c956dc106bfb936a9bf2d6fdcd8a47e525bfd761f76b84f2de0de8ccbf590bb8ecc69ead8be477d92b42da756dea30c3aa4388b76d75ac6d9ed1838c3c17e4a9925126cf06965e4b964e3d704faeaccd4caa0fee8d3ca8b75b28f3dd23d8f5eeb562d0df08460376bf915d8f02bf93ebe608c62afedb26506fb3df6af567cfb6b236e867d02e90745b5bfb8c61d66e870f61723404f7e4da963d6f04eec9b50d7bcedbd8a391f4da59fb9af49e5c0f13d23b8c5d5384f452d8f55021bd6390f4da5bfba8f49e5c3b0ae97564d74e427a9dd8b58390de16a0379787bf477eefc1dee7fdcd63d9f538f03b79bf337bafa9d526ab9ed711ea10760d7701367b3c7bf720e1dd13d8bba9ecf783c13db97605e5b02b182371dc138572d74de83bf2b27b72c4deca77b16727b16b0380c76c37dc1d3ce3767eaa60ffcd84df79dffa14e4f7d3040e1a0adf9d2e7cc77f3f03f98edb05ecfb6c47de3d93bddb53e0b927bbf6003cb37ca7f77591347b81dfce42d23c9bbd9b26a4c9fa4be1de20cddeec9b7a0077071b1fec06fd5f723d87e1b1f626742ef2de79ecbd16c2d8345db09174f6fb4e76ed681deb567d67b35daa7a3f534827935db3d8fbbc3e3d9f8dbfa680dff9f88bdcb3be43522b01873fe7e318defef615da920bd895e7753f01afaff09ce35d281f4336e1fd60febc4ba46c35f9843dbb58f82d56ff83d54da14b982cfd059be90f7420cfdb08df5dcade1b207c3780fdceebee4384ef585d101e287c3790fd9e227c97079e932babbb93da0a63d84182ad0c12e4606d4a88f79d060bef0f46debf8cd9ca8360ac4bde1f227c3f4490bf9da0f7e5ecbd6182dec3d87528c8b724f08cbc7ba820bbf09cde93eb70a1bf37dcc46a9625bc37867d7f18b827d7b1c2f76385eff97be380ac875aafe123c06f5708f5147f97d51be111021723d8753ca8a7c6b32bf0cfd1f4f93382d78be1a408724e64cfdb837bf2fe95ec7e12fbbd03c0e1cf216e47706ff0d17cb0359f4257b1f758bf20d49bdd7716dee3cfbb807b72bd5a286b5d91ef52ad7650956e37e1bdeec27baae9ea92740d3b6f5e2cc8d553d05b179e7713f27d4f8c7c1f29290323d9bb872358d9025636bbe6002c1d3c23eff610b0ae61bf8f12b046b1eb68607ba3d937a3d9bb3d6dd4b5d7b3eb750c6332f0fffc0c9e813e6ae85a86dfcb06fe0d92764363ed5b5df0b7c169cb3ae6bb2d53ac3eb9306f537b0befe74449ff46f6ee54f66d1ab827dff231c84d669a8770dfed34f67e3ab827efef66b2ed8b92e65ef64d2ec3c800f704838f456f36b186e799d7360b0599f384eff3ccef5bf176ef16f6dd51c8f7d3d9f799e03e09bc772b7b3e434867064b67af6083b7b1ebedecfd2c21dd3bd8f7338574679a78ad4308de9d081ee3383c4bc09bc5f0f8f723adfd8bf06cf67e1f704f7ebf0bbc4f7ee73e9bbbd9f55e2b5e788ee0cb60ed6ae89e48d9284e1164be8fbd7ba420f39102f798ccfd0499a75a65e6730ba1fb1dca1cabbc72bf5936a8a39a0bfe0a6e870fb0eb60f02ef7efcc15fa2073119f077f7f9ed03f9ec77e677d9bf07c61ce89f5ebc30b18de00704fbee37da081423a0b05b91622fd76de677a885d79bf6690f5f7703e7bcedbdd87c17372cffa3d49436c703f45c2fd2366de75e3edeba3a63db56565a6ed04218fa60879f49850e61e07654e33ffa6d727d8f549f6fb50a17d5a248c23b8df6d317b7f98f0bc803d1f0ebee7efc3df47d8e0e52953d7a1dcf7f9b4c941bbfee6f37613a2603c83b43f4b84366509c3dc29b43dc08f1f663ed770217856c8be7b45a8830b857a80bdd79eb7a1fb117feeb3eccac73c23c13db9f2314db6f09c8f3172c03db93ec7d25fca7e1f85fcbe4cf8fd79f6bcc89abff49ecb41ee97b3df47031cfe9c5c57b0dfc70039f8738233d03afeac2657b155ae7031f88edc9758f143c3c173723f4298631d6895bf2afdb1ec9e8d67c32bd973d69f0f0d139e1f213c2f0365f645f6ac14c1e0cfc70bcf5701bbe2e94c10eaf995560cae17bd17e521df4f0475347fceefc9ef9340fdc66520cf270bdfad62d772f6fb14700fbeafc29d2a7c5fc19e4fb3e6337d4eae95423e5782fc23f7ab053b19089e93eb1a211f93c173583e787df392bc5eed703da8834979bd4dc05b0bca83f17dfbfdecf75a92dfc9f331e039b9ae63bfe7827bf8fb7af67b9ed54f137a59284f4390ef5708e57dba203f7f3e03a407d6008437b0df67827bc8cb2ba6ae3db8cfa103fbfd51f379a76da0fe349e77ea24c83d4128ffafb2fb8d4239e5edce5142bbc39f6f12fa0b9b04fdc4f66899508f2d15eaab8d427d374b486fb390de6676dd626de7aad2db2ab47f5bc07398ded1ecbd2381bea09eadfa7d84c02fc64f720c1e0479abd25b2afc6ea75de679fc2093ed9528df303be9d640f08b1e2bc8f99af0fb7191fe2b6febc2c7823531e49a03fc6fe4fa3a7bef18863d5b5857743ca8678f07ef92eb09ec9b39e01eca9d237c77227b7f2eb8d7d8da1ae8afda66e274e921bc779a6057a7b1eb490c771eb8c770c9f554a0d3a92cadadc2b7270b982783f72126fb3e958f03d9dc6cd7b9c277a70878a7b0eb76016f3bfb7eb8f0de99c2f77c5ee2aceaf39be1b3817e67816720bd2a1f1b5bd3123e23623b5def1564df21a4cdd64a84cf0169f3b9ad7341dae780673c0d98f61956eeaaf279a790cfbc9fc9d6d884f9fcc47c704faee70b5c9ecff00b84f72e60df2f00f7d0e6194ed2420187e7f19b02de85c238e942f6de5b114ef39384b9f3d301776239e1f359f9d6e7556bf6f89ce02213bb73b18dfa670c68bbc9b78bedaff30a5f02f2f41256c6f97a9951c83be47a31787631fb8ecf81bc0d9ec33ce363fd8bc0b71799df76ef247c7b29d3a500dcf3f7e178857ddf63aef0fd2e21dff89cc665ec79a1c0fb65ec7ab975ee227c39f025927bee2f5f02eea17ded66bf2f05f744bf0351f284cbcc7dc5cbc03d58db17be92fd5e04ee0decc32b059cab01bfcc7f1cdec3be5d0eee21666f017367147939a7d70a6dd6dbe039b95e67ede3557d77bdd007b90e3c27d76bac7dbf30f3fd86de61f737b0df57807b6803ef5ae509bd1729ab3dd6d9281bef817215cbf7130d67a90487f5a77b7c6dcad3332986dfe27d97fc163722fda39b90fed15ea17f7423781ffe3e42c197d65ab27ee04376fd00591ff011323fc5e7b26e06981fc778f716f0ee27c8bb9fb2776f13e61918ffe15b25f36a3c6f3e13f2e6f3187967c707d63aca3cdeede0b703883e3ced2f84b4bf8c219b98d61d20adaf84b4beb6ce1d84be11c646d1f4e4e3a27da0ceda6796913436f608dfc5d22b06f7e45b367717be5bf01d737ff6b7000fd617dcaf7e2748f34e96e62b425d758f5057ddc37e67f370e17b85baea5e30c7cf7161da2c9d8cbd607e90ebc2f9e56371de47ba4f6853ee3331d29f1464bd5f90f57e41d60704591fb0ca40e7cfe198f741960e5f1bf43d98af21f73f0836f4a3751e23638fa00f5fabf690a0cf438c139ececf516c93e3d602eff3fd0fbf09e9fd11a9f757f3f5587cef02db8b1062ebf8c30f5bd7738599ff3af44b04a32445489fcdb966a5803c0476caf385f20feb589bf99bc9d750ff25f0f1b7958faaf4459e1f1470597e6671dfd13f02eebfd63a80eec431f08b5a81746a45bed7d87a694d8fcdbb1676ce3b4f3f73978d7a2555e8973dc2d22901f706177d93acef6b7c4e9bf7a557827bf0be56cbfa5df831f67ea9294f98cd3768c9d16db92ff33b68b5a397a5aaf7389ff56d70ceea13ad9e95fbf0e302e78fe3f37a55e9b2f6586ba4906e4321dd2784749fb0912e5bb7a93551b7318da7f3a490ee93b88df174a3d996761068bbea5a7d407d591f553b38feb65fe37da096f6d68969cdc1784d90a71fab8bb4160efa227cfcff94309e7b8aa5f181f0ded3c2d8eb6964ced34e9acf08693e63a6d93f4f786fbf90e67e643e331a8f2d4d1b18b657c07e5690e15926c327c27bcf0932f0b99be7413fe379f64e99fd3a4ce36b125f10eaf3174c3906305fbfd69ab5cfef98d701079076e14501e74586c3fc455a1bf3fb4921398ec6d794bf24e0bcc470d8fb5a5bf3fb891cb71d82fb1dc0afab50b7f3f6f365418e974d3906321f8f7628e365bb791d84b597af0838af301cd65fd40e03f293fb94e8b8553cbd2ae0be6ae2f2f7b5f6024f1d84743acad3e17611376faf0972bdc6e462637fad139363ae791ddc1fb1cbd7059cd7190e5ba3a075067649eebbc4c0e5bcbd21e0be61e2f2f7b5ae803788f393435ede14d27d93a5cbc6335a2a909fdc776372307b185286f0f49680fb96893b84bfd75de0e970219d1ed1d3a9b2e3b78574de66e9b0f7b59e721ccebb6ddef8be9a77587aabc03d49ef37ebf85feb157dfff5503e27f6ae752f61f85d901e795e6e4d5feb654d5f792fdb58b09ec24e7bdb5bb2b69ea45f21f0f29ec0cb7b262fc3987f554b63eb0b27803da2e4bdf785f6e67df61df3c568e992ef483a95acfde2fe5beec3fc40c0fbc02a077f2fc4f7f47c087868278cf5a3f1c8f7897c54ddcfa36544df27a265da4f47637de2216c1ca0f531ef170db7f16d5fe1db7ee67dc1041bfa715fdac7423bff711cfd1bd63f1bb907f1b77c16e150eb2ff7ed687cedc901c157c5dad5f0e7d675fa61e68f0a7f0ab007826731f6a785bf00dfb1750bda48819bafc09ed947c133f2ee10f03df31b6a604d3dc527cf8601fb617e136d7804379b8fc1f89a8a2f016eb6551f2d27f2ddd4e108d75f83ef4759b9d6468334c8fd9808de847d82ee9f0876f109d2076571239256dbb057bebee11ba10c7f63da4f362b0bd973ad3e676d70f4baac0af75b26cb1a700fc7cddf096dc9772cddfd6ae971dcf0f70c6fad20075fcff803fb7d9df03e5b9fa18d057643ae4744f2634db1352d8def41f809cca9f17b72fdd12a4b950c3f0adc6cb0a619fe19d80b5bbba44d96efd30eff8f616c04df92eb2fd6e7da94d87a55e9f3ab80f93f80c97f27d7dfd87b4759f5d2601c1596cfda44500e59bbad4d8ac8b0963f63eb99b469c2da7819765e756cbe765e8a9d1b056b7a752c6d5814ac3c1c8bfb782c58c3a3604d07fb8e08e6ef56fef9bea7f01fc2f396665919fdb5b0b6e8cf481a6342820ff64f900fbf035cfe1bb9fe153b5fb519927cfd0bd7b10a6370fc310134eeafff5ba8abfe367918f38199e6d83a42dd3133465dc571ff11e6f6ff11eaaa7f85baea5f33ddb1ebd4d2e3b8a49ab6d455b3c0737265f36549ac7dd4783ffdc82875d33fd6b2acebd6ba49676d891eb6a65d956658e06283205b2d6b1ba4f36b9275debbeafddad67941bdb63cae8c5ec71a57466779a8d715d263bceac9427a8c131dec65d5d89a136dbeb5eed416203c827a55af5f9d5bbd3e480372cce2ef246d06f7e4ca62ea246db1721bab7ed41b46a92bb8cf21df5a3f4ab155ebc74551b064f5e3ec28588b712c69fd38270a5681b5be1bff8e553fbd51e49bf1df5b79adfa4d8bc443d293c16fe4da58c83fe613d69bc4ce3f593da837c175a9c298e9a01e64eb71f4a6d5e76eb542f00e1c07f1df97288cb76680358575c13e6d92f641d63a5867f17a26565adf0b1dccd2dbaa30272d4bef60213d16b765e227427a07d94f4f5b6af58bf1b5907a33a1ce69a6eeebadc26a2e6035571fcb696c4d8dc6ea26ad386257eb789bcbc6317a0b609b454239e1f1498e06f7068793d3c0b7e03bf4fd3c5ce62a0c36fed25b59db4c9dc53e99cc7ce05a89a01b883b9437c5bc4e592660b335f87a6b019b8d79a7f0317f9980bd2a825d98846033df8fde46c06ec3b0395685805d19c15e3ad68a1d356fd9b8485b6d236f41dc2ffd106bbdacb715f2aaad29efd450947ecf5a216d919f5836ced643e8dc87730cb82778ac5fa41f6a5d1349ef896cbbac3821be9ff35830b626eff3fdf8c709e9f2e7c72bc8da5e90b5bd5046f8defd13c03d6c4f3a58e77975b6de3af737211dee43677d759dd7633c8ecc89e0def87eda4526efd3d8fed7dc17acdf69dc377992d517a683751d1aebabe95d24feb1a362f8c73609e988bf6f16da0dfe9cb78d422c199df98c75104b86fe4db83a89e9b847c219cf7fc041957f777df436b32aaf647949b8ed24f4b93600bc29f6d2885a96b7823a8c5c8f91f4c58e8d94b10d3c4d5e66414c2cedb82858d36d6081583adaf1009fdc9f18f96e23af23b9bd83b836da49002bd6773d709ba398516c4eef11a7cdf5126c8eed93d77b029beb69dadc8c3a51eac19371aeabfaaad1b8063175b453acb10bf474a1fd67fbddb56d11bc4d3caf59ac233d0d70c9ebc2d30057e49a615d73a0a709cff99a85d3adbf6bdb0137dc1ece04cf785e83bd013a5bbbab9d0d6466be578dadc39fc9e6da66b2b97a8dcf419c0b6c00ca3f5e909ffbb9ce016b357609ba817e37fd3bd63c37b797f362d8e1ce1876783e9043f6fb05824ff7c228f634c3863d81588a3adbd7aa6759e3b1e96c5d85ded71aaf4c677beef57ed678653aaf63fb58e3955571dbc71ab3a6ea797f10e78fdfcbc6ef0384f13b9b3bd0070af6cff519648df7a6337fbf3e44d087f96ff5a1823ecc8fa90f16f419283ce77d918b04bd86097a71b9865be3c2e9cc6fa78f14e462f3127ab620179b1fd04708728d10daa61ca17f996396a3d9fb401c30f27c54c44e66ef07f1bae06f5c1f2166a13e0a8f59c86d53e7e3445e075e2cd8eea5c076f9376323785bf6011cf20edbbfa18f16fa519cf72304de8fb0eecfd1c759e30fe8e304bc53accf35be7ff7546b7c9eb9dbac716ef489c2f871227baf93246e8ea1d7dc2c791c1f9eae6a3c9d6adfb139feb9bb85df85783ada2e41dfcb22dccffdcbcaadd29892ed47d1270bfeb9c948f99e24946fb62750bbdcdadee87c8cbc4d4887c75a39cdfabc9abe97011c38e63e1dc1eb6495479f2ae83355d0639a605fd304b9cf10f07285fa2b370eaeafb48e5ba3cdbde8d3059f30f3db6b57cbe77ff419c23ccf350a69cdb47eabf3b6690ff85d63b1c2c17c803edda61f7356149fdeb551b0247e4bed8a2858d745c192f92d7747c1ba5ec092cdc746cb6bb67f45e3fe8d1bf0796fed6676bd517807cc89eb4756ff4e3f127022bc4f7563efcfb7e17fa0fb0c09e66ca17e9c6dd68ff97cadd02dd6f7b5db403e73df2dd82fa2f3b90ab0af43e7fef83b23321edd1f7ccbb1ea826f39565df02dc7e2cf6e0532d7b5cabf88d72bfb0459af90c8aac89d7697d56714b5accd17ca35db43a2dd8394eb0542b9be4f21ad8542b9e6f31df782dfc9355f28d7f36d96ebfc28e587ed09d11e10e627e6bb303fc1d7c73d645dff21c396cd3154adf590613f2c60a996fb47d8f78b04df025917f2b5e09f5904e45c6fda6901f3ff14ec8d324e7dd4eafba1754b5d21dd1b23e92eccb321375b3ba62f16fa818f21ed08fca6403247b658b0c542eb3e7f9dc5e7298cb2ff537bdc9a8eb21f05c412d29e888265c78fb214603d09f063f94340fc7eed298015ebbb22891fe56980291b7f3e03be95fdbe1ff1a3b01848fa0ac18fc2fdcdcb6dae5764733e7ab16043cf217516fca6446243627db652b0a195a60d2d7d278a0d3d6f4d47d9a7530ade677e48ed45792cf1a8f95926c9cf97c06fb2fc7a19c820fbfd15243f598c17bd5cc84f3ea7b1cae6ba5b3e96adb4ee2bd3849821faeaea3143f44a2166081fb7b058211a1fe755c86386e870fcba16bc4baeeb043b5867dac18afdd6b4aabe5b6f8d19a2b3355efa06a1afc362efac28b3bea7b1580d3a8bfdc2cf70d1797bb9c66aef553a6c023a6c02ef92eb664187cd42da4709df6d1174e0becfad820e5b4d9ce59f98f9b19ced995d3156f8ee68201bb7751ebfe54c704f644a1330d8fbda5b56fbe3b256f9363659634c2c2fb0ae8fd137023b14f36ca3357e057f5e85bdd68a5db44bc011f3649c9037fcf7b36cd46b6c9fcb4abe4f82edf7d7de05fbca08e6b1565fa6c6f6f7ebc701ae8f05cfc8bb67db48fffde8e96b2c468a7ebc903ef3c1ea2780f48f07cfc8bb3b6ca4ff610cfdb98fec4421fd8fd8f39340fa278267e4dd736ca4ff19bbb2397eed40a4be3996fb99f93a5cb6f658fbd81a9f336a3dc77cce3a8fd7531fdc93eb29d6d815f4de28172be75abfa77cf0b37278ff84ef7f3f02e8c0f702f2b5e15f087dec8fadb65ff53befc79d83fccefb315fc97faf6aefbe467ee7e3fe6f84dfbf15d2e13efb7311fd31beb9df10c44ed299bf4ee37e1a166f29e93c700f7f3f4df85dfc9ec7bdd909eec9f50c612c24e26e17bee375fc99c277a709df9d257cc7ea13fd6ce1bb33001eff9d5c770863af1dd6315a88c75b3c5f712e92d7a3e708edc339a6edae62eb1e56b58a8d11e2eded05c2732c1608ff9dcf87b48e92065b6ba5713fdcb9ec9b0bc13df99dc500d0cfb3c6c5d1b91dec14d6c4725cb6de543f5fc0657c6a7c1ee5020197e9aa5f88e0f2ef2e12bee3f30b170bf9bf13e0f1dfb9bf06e4bf94a393857ec8a5423f84c52cd241fc248d8fb77781b2c67df29721fd95cbc1bb978377c9f50aa1bf7285694795fb056cfedd6ea1bfc27d6d7c9e87c703ba48e8135c0564b8123c23d7abad7189e8bd2143c50ba62d57ee46d2da23f483f70869f2384017837ba2db2cebfb3a8f437409b8e71c439c4b051c412efd3aa023f359864e8ab1a6e172a1ff73b9b5ff53d14c880574521ceb17fe10ece17a618dcbf5a63e6bd2aceff37645bfc11acb40bfc1fabef68bf53bfd466b2c03fd46c97efe68ebb57fb5f95e3d9beffd66f3bddfedbd27fdf60881e39b048e6f32395b7baff57dde36eb7b058ef70aefff61fd4ebf59e098fb9aff0463eb68bacea899f7aafa426ebd27936582500fdc22d403b798dcae5b687d5fbfd55a7feab7021f23c45928e01c10706e13706e13caf8edc2dce9ed423a77587fafc2bd03f837c9ef51e29868c50207770a1cb03842eb0bacefebfb04d9f709767da7c001c3d9c0f720ff65c5b3f8e3726df865ee12d60d301fbc7e8f303fcfc766f70af3f3dc077eb7303f7f37586bc9d366fbffb4bf85df304e8f1138bd4fe094c5ddd9b0cffabe7ebfc0e9fd800788b3d08ab371ac80f38080f38060570f0a76f5a090ce43825d1d039ef33c20bfb3f39036ceb5f607b5a363f0c3c7800f0b6b59b8effd51210fb97fff31210f996f5a7f44c8c347843ee3e3421fed71a1cff984351663957c4f0a36c6fdbe4f0bf271bfec33827cdc1ffb9420df53421f72bf20df7ea1effaac553efd09f09c5c793c90cb05f99f17d6b0305fa8fea2203f8b9faebf24c8cffd932f08f2bf20c8ffb220ff73e039f78102f9a436b10dac7d23ef5ea15897f3bef02bc27887c5d6d8f4b369ab9bdb0aeb3ccf8861ab1cf75561bd0af7b1f2d816bbc13d5c8ffdbad06f66312b36ef8e538e370539de14ca368f657125b827d7b7045e584c882d0dd8fa9d2c357938aec6c7c75709cfcfb28efdaa9ef371f1d5c29ade33e268bf2f15ead9b7857a96c5a3d89a627d5f7f47a81fdf11daaeb7857a96e11ccdf7f3fc63c5536ebbde15ca25f3a3e9ef0be592c5e4d43f10ca25f7ebbd672d9755fa7d28e8f79ea0df87608d16798fc5993b5a5c177d490c9b64fd15fd23c1b63e627c2d63b8fb85f601c6937ca2fafa80aa67774ad665df6a53a68f8572c26227e89f08e5f513a1bc7e2a94d74f4d5d8ed95e033a7c2ee8f0b950d63f13ca3af393ea0784fc38c074f88df94ce7264e172e93c6fbb2d708cf791fb6a5f09cf71faf15ea895be3a82778bffb5f704fb0afb3feaef3d8a5d7837b72fd923dbf41789ffb596f04f7e4fa35f23ef7abde04eec9f55bebfbe1104897e3f1f7b88f1b9e1d6c777ccff72b6b1887b5343a961f1b2a0ce587ca42bde987dd28fbd342d9a15e462929089586561a7f651bd712e35ff38e3fe75ff608a5c4f886ee3ba69a92c09534d9cad07ae3d342333e5aae7153613c2a34de2718e38c6f2a4399a10c8a5d62fcb6d8c0290f15878a428b42ad69dc409e7a36fd7fbef1461105dd9b542d89bbaa2541842b36fe2d343eab343e24425618cfc7184f561aff951bcf161bf7138c2bf96a61283d34c7f8d78a32c348b638b4dab8cb33fe5f46f1161a62577f33964af38ca7e237d30ccc9554ba12e30927b4ccb82fa612e6d2ffafa1921652e9c719cf2a8d67449ba5a191c6751d4dadbaeee5c6dfb234f318657219e7293dd76858499e4d39863644fade748ee6ff72a42672848605d89b6416c3a4ded493774ab59cf83f0ecd4aa6169d483631d229659ae55946a8375d1f3cad1a83ea3a56d7506edf26df7d43595175913343b2fb10aa4124879ffef3f76967ccd8ddbc9655b5cc506fbac4b96d0cd5a2114bf05269845015156ad54a4a23828af46b32fa6957293781f49bef15d1f7d4332099367ed60ca06131a71ac24ca1f55ca401cd3580495265064c2ff0c60276ad966369c9ec40975449931a4b708cf6b08cf63a89a63d763d138df63a557a4568a7c142a50cd645188c2503c6a02e6390f6c74a3c6570baf1ccec108da2bf16d15694d485e5060b04711ab5a3028a49fe235a615f712e271ad70243ca625ad346639d4cc3f4145897335e9f329e4e87655ecb8ce552922c97e84c52718de4d204f697b779d4c0761e35a4799416258fdc9218cba15ab21ca26b778a6ab01c913eea12aa4589cb79d3c876de34a679d33366f971222b962bc9925c4923b9d244267e5d417c2239025c1b036eea10b80e067c9043e0ba18f0c10e81eb61c0cd1c02d7971529baac382f818d3bf9ad94a21013556fde496c9dc3c5e65d5e525ad092d2aa9a72b145c0ecbf818c42ead19deb2985d5dd2dfcaf91c6b7cb0c3dd6d031f46a56038bef47a7b4a5edbaa715653445ea3452930863b8a1ccfa1bb258b731adbf156efd8db062d5c661b16a2c0326867288d45245e41638721399b5354afc20c81c4714c71851cb6d8b0c290fb3dd1b6f878e67a2cb8059535359de346291d2aa8b357c78b5ec698767cf41b2eca1d6b0ac46ba28130d4d2a99eb8ce8e46e07e530db95440acdc41e513a284e25c5b2fb605976931c696fa78ca7e059dd4c96d5b47816d4485667d3966b8df1ff8aa8e49151acfd2cee603b8b3bd22c4e8d92c5f14a88656d73590e34adb9c2e6ed48ad93ed9ce81cb3b07935426b81b57b5d24ed9ea6d2eeb5c490bb3a456e25b3225a8f97d650954dbe5c56351b52c4f2295a1e4d31de277292742b146c2ad5b64d75a3369519b50277536eccc25a6376d0dd69cfaa8dcc0e68ebb12821fdf8dc504e68bc216204ad94b244d2a9a07332e4bd291473059d9589f4a1f9b7d173fb70dbb9dd83e67617d0a777433a2c4f0f91314f9bd7253e627eb2d1a75ccee626894c2b8dbf5473a0a7ed1ce84573a0bb720ed89312cb89b6d848abb7c391563b193071f0a4c92644c80fd368f69159fdb5646828a6551b4feb504c8974874a1c26b3d3e63533ff17df3c080912d8d2f6c82b33eaf49dea2c488a2c5b087b59985062d664e259d31e03efe3027807accde9ebb4cde928436eca22a3c634d5ce3870274ce4fe4ebb4b9db1e1f3006c54af307cee82810f9480d77bbde36015f0aeb2e24b7bac4b6ba4a3378355d3a4e864ba3c582051157581afdeb40f7d78942e9c3389480107ffab25af5406d34ac53b29b0ea2715ab2186b8504374c3c087ba00de1d031fe602f8e132705228864b2c28ada5258759368ba90dc653eb81750346d8a9efa2b4f93db18a63a40bb5522fac32cd765afff79655497440bada17eec229f4fb32fa84bc3dd278564a7bf8ee565739b6fbc5a368f53140c191e89e0e58d59226cb453a505d98c0f1cb48ea572b0463b162e3c97a8a3782a6208e03f817d1f366b4edbc1943f3a653b5314bfc92618ca7632572acd312998155b6e35ca86c3331b18f702a7616568d8ff7a21aef83e931c1a91e7d31e4894e91fbc90a29f5f9cc49b09321875650f9e02fbea059d59d30a94a27d0cba3de94c3a48e039594597c60f39fda48676e0a2df1ce13c3ca797f59a6b5f57ee59c6afd75385b5f435a15735692eb1c5f9d3b55d67da73e94f4386a5775e974ba6e9f78982a68eb68aec597d61be6ab84c64a0a6abeda9b3a617a4569a6a71a4af1867a25fbcae18883556472539d56b5c4d07bfa30731e80d5d2b95ed4d203b1d4f2bc486d90aca8528fd9f21ae9caba615fd10be974198d8756a731da687806b54aafcb09668f83657996e20f8f4886cbb9355356a5b657f1886438a89f90ec9fa5e40cc9702de387c8aa0622e891329b6e1fa3569885d70a43b18466bb9cd0302ca1392e27341cf334cc75c1d33002039fe702f8480c7cbe7ce1d33015f06cace3bec069c73d47564975a8b9dd13e2fcf61c56edb83d37bf50d6b9efc8fb80de4b4783c4b38d82a15ec63fd8206051cccd1bee09855568a330f35bec74be6334e61028903804fe35fe5171088c91d9369dbb59e10bdbf6c2ae0b6dfb939650c3eaad6058ee1ad558cca8963aadd3c6c9903bf05320c4d68a147a1ae23d0594c668ffd4aedea22dc28539426684749aafb0068db0d4f50574cb653dc02e3ce281fbd2a4fcf463f9b0943ffff86368ca393b760c35326feb30a40a5d412ddd1b3130eb1e8fcdc5143b9c8b9980159b12a7c566a20cb90e0b34dcda4e5d5c17079f84758f4a5de87b4d9695313a213d35814b47ec6c8297979d3259bd442c7715b5dc3e512c37d7f878195b69ea567d2d2d44e55494e6d5f88cba431dc9af29980d5738b5e1a9989955bab0c4619acccce8ccf7f41ada1d3227467c0ab9b9ad96f57de9247beba8fb40b0d448f0ff3a3a3b9d15fe8398d25a6a4af1a58599542ed6915ce7c2cc521e06be5e02ae29824fc7c03748c0c38ae033b092b6d169cf7d2656d28e72a1429f8535459b244dd1c2b0043c4a537424c6c966a7b5cf6c8c932d524eb62af900e6c8c0c94afaad767a14dd70e0b93260d27ad2c0a3027037594e96e3e0f330b28f714af67c19f2c1ec6ce7987cf4c08117c8aa79ba4ce8e8046f399547051b63684552838b91ab2f5ae661a1c84adf25ac83b0924a113b9a58f4e6e33859f341573a8d52886de6bd16e46c967aadf066e804da0cf94b66ac795b8895d1132565b4876219cd9781376351ef6396a45e38f0221970777e0483d8e12526744a24cfc4744ec0d3598cb173aa849d5e8aec1460137ddbbc98e82bc4523bcd8bd496c852231db3d3ede4fc181c78a9ac0eed91e88dd4b09caa7793cfb0ed4adb8e6ca28e953e56da97615dc2335de8cc16617dabb3247dabd464b5bed5720cfc6c59c74d117c055636767851368a3155ce91a8425c3b2aaa9460aa9ceb852a2b317b3acf057b2ac578da29e1a999224f6518f8f912f03714c15761e01748c03bd551032fc7f6495de8709f540526f54512a9772b4a5d89b92d2f96ed244ba27bd73686d28c71998a0f733596ca25b2541a5a02f82db0d4aa154ae9aec1a8bb5442ddbeba6ad4adc5c29dec7218ee641d067c9943e0f5181d974be8185b4f8d8e0d58dfef0aacef47628887697b7a65cc753bb29eb8baeb5ea58fb91123eb2a0959e4cc4315b28ec2c0af96801f50cc894d585bb3c78bb66633d6d65ce3425bb305ab38ae95551ccde80ed875b4821a685c330c2a36d2a57beb8c44c8df19557fab54235bb1dcba4ed6e2299ac2d118f8f512f0e314c18fc1c06f9080ffac087e2c963937ca32a72e754292962325944ab36553a89b52361c87a57713b6eb996cad5d61b42366303295b48ec7d2da2b4bab099dc52e34de5c101a41a7644a8de6219bbe50ccc3cb2aa47e0296facdb2d45bd375410b0c6e17c49482bc574edf5191e744cc8c6e919851654335333a09eb38ddeab0e374b26c4cdab3e6025c56f71cb93b237f9bedf1ebed31035c3a97151beb9e82f513eec0fa0977c6e5233a152b43fbb012cccb5019d5789ca1c9027a974fd7ab972895986d58ea77c9526f542df5f8533e0d4bf96e59ca0d40cac47fb9946d2a21d6ae92eae958aede83e5eabd71e5ea199876f7c5cad5eafaa9d7cbdbb1d4ef97a5de14b40ad69688871229346c4c25fd33b1f41fc002a18bed022e477c2dc35998440f621d34b92493e88e977879391b93e2219914ed62f2525d9af8d8d981b59b0fcb1c0e8dd5dacd7330a51f91295dcb32a85651e25c4c8947254a4c5054e23c4c89c730ff430ead13d72aa9b0139b2f785c325fd05971bee07c8c9f2724fc3468a2c6cf05d858eec92863b9adc322ff531acb5d88a9f294ccd5a4a8ca45d864f9d3d2a53a570c52992cbf18037f46025e7ad0b10355c02fc18c74bfcc48ebd0f5a5a43a994957665586962919eba5586acf620db8991a8f1837a16ac39d4aaabbb0549fc31a563355710d5a7ca95f8619def312c3eb7f909ae15d8ea9f602e67d2c336afd718c546bf80315a5aec0d27d51966e0bd03696d3b0ce6648c9027058432e0dfb5ca04cef6e4c929764921c26691fed48145f1b792526dbcb58ee58655940c761a6342ae95e85d518afb8b09ee96a4ca957654a2553c2e3ab2df66029bd86d1b7b86aedf3184a5c8191ae39c45349f71a2cddd7b1742b68a4f2252cbb16d3e122d9e0bb4229dd6bb1f5456f385d5f741dd6d4bee985dbf47aacc67b4b52e36d6fa156e3dd8081bf2d017f4711fc46995ba557cd6d731d6ffc7f3d452e604b6edc73a990a36ddb097cd5e58b5da5ae957763ee4175262fe656b909cbf2f724597e544bb52cdf2b032759febe849f3a9c1f318177f1046ec6a4ff4022fdf78ad2df82817f289bd86fa5067e2b46cd47126ab6c441cd6db2e246235d6da8d1d58966d8ca425aa5e757e14da2edff5a6ab82b0c735ec39a963ceacb327b134ed7237e2c613614b5507e420be5501b2bffbcd40a2bbab7634de9a7d8403e9b6d2955eb05de8199ea6712425b68eaa67a27a6c8e7ee87a5dd87297340a24cd33894b90beb1f7e21ed1fee523ad8e26eac4afa5252255ddf5aad4aba47064ed6e27d656742653b0e7c2f96bf5fcbf2d7dc3a54484fc932a3deac61ad5c4aa86768a8f17f739e7689f27cec7d58de7f23c9fb83e2c8fbfbb1ecf956b6aaad8d5af63c80c556f8cee5d80a0f625eafefb1ed673fc4dc7b3e8dadbd70e30034051fdb435861fc515218b332578c52298c0f63e03f494b7a6d25f0473053fa59624a6ddbaa99d2a3d860e87f4e07438f61f339bf60f339bfc6359ff33856a5fc86cd1f64538fc34adaeae6b22859e4dcdc52e6da26f376c58a7b6c9fc0867ebf7b31f47b52961a71bafc613f0492ccb13b034ff229cc0aff9458614a3b352b7c1ab3c2bf9c5ae13398d87f4bc49eab28f67e0cfc1fd9da1845f06765bd761ad974554042edc17dbeea21f7fe55eea3937d94e6d6e7c4e980f5c89fc3ea034db3531f28ceba3c8fd5b5610da96b75e3877a8d55ebda17b0ba364943bae713a9a7b582c6de9aa554a3be88a555cb4e5a472aa5f51256909335c9eaa4f66a05f965ac73505babde3918f4febf4a5b3c5fc124af2391fcfa0e6a92bf8a495e57933ab847aa48fe1a26793d89e4299dd5247f5d5679a67beff288163490a0aca7154d91f1cb527a1f7116e4d34e71ec7082f146f9a9af55af4eb5a8d569038dbb3c6a522bac827d031b9a34d4aa4fc837549c907f13b3cd4612dbdc9baa669b6fc96c33a3e60273bab1303ebaed35d66c780daec4197b5b961d242a57133bc04b70e077b00aaea9a4827b76ff84212a15dcbb98111d2431a2b93dd48ce83d8c92831d52f23ed695688675259a6bf10cdb3ec0d26981a5d332ae743ec4ba11adb428cb28d35d5846f91196726b59ca078129f35cea0e9e40bb98954639aca0d3d4eaebae3ec62468239380ec0f9a6ca460a6034f8e5349f3132ccd4370be89137c05ad81d6d23aa69cae4229524cf9532ce5b6b2945b5a1676e212c4b724e1334c96763259ead17abf925a9999dfe6204425c5cfb1f27428569e0e8bab3c1dc0344bd1902d1b119bca61277fa8e8f505965e7b0d5939358a2eec888fc52f31163b602c768c8bc5af30ad3a69e822e4c8940d5f563ad2f879090b18a1a2e5d758cbd459d232edebadd6327d83817791803fa908fe2dd660779537d843551aecefb07d68a99af37d68df63b47493d0b22b5d8d961f30b3ed8e99ede17199ed8f98d9f6d090f568b8af75146d5249d3168fbff5272caf7a6ace4360fd8cd1d90ba3b3775c74fe0fb38934894deccc50b3895fb039bc74cdd91cdeaf981164c88c809f7a5d688c44cad8e882f4665268dfc6348794d040369747b6f69167296c7b5f8ad16610a7ce2216be3d9bae135d4cdb6ab505febf615c674ab8ce53e4fa778c922ccd59e8ce3fd063345da892fec4a4ee2bcbc8c6554d79a46368764d73e82e41b5aee95f5811eb8715b1fe7115b1bf311d0768c88473acd9a1f83a16ff60fa0ec4f41d1497beff62fa0ec63ac1766ae8f8740e6948911b222972db33d58a9c86810f958037c952030f6b088bc334643750fc559c0aa33aa6f47089d2fb15194dd290ba6684a4aed115eb9a5a12f0de99890e843a1d1c951c5f20d49192e2da3b4b1e08d54e6ad9065e9d241608b5599f50e851036c9b71fd3a0b8d4297a3c983a1da490f73a7266b48fd340aab9f46c7553fd5c6d21983a53336ae74ea6025659ca4a41cd747ada4d4c5aa87233464733dac64f16a819c2a134fb5500f93673ce66d8854ede57175bfeb63f44e90d03b5c91de0618f84409f85f8ae00d31ae2661d38af12f0a698429325936ce5654a431063e453667d9570dbc09063e55029ea608de14cb82691ab2f143deef9caee84a3a48435699e46a0e57991c8cd56a7958ad363dae5aad19a6c10ca71a34c790673a456e8171330be3e6c8b8b8698999ec6c89c96eedaf66b2adb0bed11ccd7938efd698e4736595e90035c9db60e0f324e02729821f82d132df852e635b4cf20512c95f5094bc1d06be5002fe8122f8a11878be047cd64035f0c330f0453247a222780a06be5836f1ad08de1e032f9080ffa608de01032f9480d719a406de11035f22016fa508de09035f2a1b262b8277c6c09749c0df5004ef82811749c00f288277c5c097cbba2083d5c05331f01512f03d8ae0dd30f06259b74f11bc3b065e220127e74aaa801f8e81af94395514c17b60e0a5b2055f8ae03d31f03209f83245f05e58e76895d3ce516f4cec72991d0e55133b0d03af90807faf089e8e81574ac05387a98167602384d55a94184d7d5c8abc9589a5be069be2e60b09e2db55978551b95646e570352afb60e0eb24e0fd15c1fb62e0eb25e0058ae0fd30f00db201ae22787f2c8f3762796c8df3344e71fa690096de51b11603554f37be252903313a37c9060223d4e81c84816f96d5ed23d5c007cb7cc77d6a2e0a233c023a33e6d9a9eacb25b76876a3306ed5624561742e2b7a64b52ccb492e1e6d67ceb37e9423aab141e5312ecc790ed3907d19c76a1eecd31a8ea5769c17a98dc0883bde85351123b1127e826c598f62f5918d819f281b302b561f39182d27b9604fa3b07afd646c0eddacc917d37256c2a200c4d7a68cc6583b45d67954646d8c0c9c54baa7da29dea462c20ef1c5a4de26eb90e4a8493d0ecbebd35cc8eb2330f0d35d001f8fd1728684961da3d468998065e67687993911a3e44c17289984d59e6779517b4ec632e06c4906dcab68975330f01db281b562ee4ec5c0cf918df314c1a761e0e7ca36982982e762e0e749c0078f5603cfc3c077ca3abc8ae0d331f0f325e0c563d4c06760a5ea0249a94a522c5533b1bae0428775c12c1930996cbec80e70c328a76f62745cec4225331bab642ef1a2929983a576a917a9cdc58c7497c4485f19ab66a4f330f0cb24e0b71da1063e1f03bf5c3655304e0d7c01067e85ccb3ac28f9420c7cb7aca61faf069e8f815f295bffa528f9220cfc2ad97e3545c91763e057cbfaaa8ae00518f81e99a768821a7821d6c3bf06efe19b9e9b021643207e4fe4122ced6bb19512912d4d66e0a482aa48062ae92ec508bd4e66c493d4085d863540d73b6c808ab076e20617da89e5182537ca8a9e22252b30c96f7241f2628cefbd0ef92ec128b95956a627ab51b21293fa16875297625cdfeac29a85328c92db2494244d513c7c10a3e4768794946394dce1427fb302a3e44e09250b1529a9c428d9e79092d518f05d0e81d76074dc2d2b348a74acc5c0ef9180779aaa06be0eb3927b659554929a95acc7a658ef733ac5ba0113fb7e17ead68db2a989be646a626e0283349113b57933df8b6edd58411b7f2c7851668c09880724b14292a3c60a79904e44a4540bbda42e1936dd7014d6157a28da9470ba4b53c29bb0d41fd6621c1a546499b68b2ff5cd58ea8f60ebbb79ea63e92a6fd3bdac92e2162cc54765291e6ce978564f39be09cbad5865f698cc7f93a778662236107fdc8b81f831586bf284c3d6e458ac727bd285caed382c039e92b90f1433e0780cfc69097883e96ae02760b43ce3022d2762a6b3df0bd33909e3e959094f398a3c9d8c813f275b793c430dfc140cfc79d9ba4645f05365e02488d20b0e630d6dc3805f74087c1a56a5be24ab52f5aab34795ced0c3287f59662c33d5283f03eba2bde2b48bb61de3fc55879c9f8915d5d7bc28aa6761a9bdee456a6763a9bde1456a3b30cb7a53366c51b4ac73342454c15b0e43159c8b95b9b76565ae1bd9d5c8ce0132f7efe6d1e076663826d2851b4dbbe325ac64a6b0ae65640b1fe9ce9a1dbe81716de43b0f93f79d68ddcc0c973ab93bb1d4df8dd5c576a3937b3e96fa7b586c11b18bbd926dac344f5c8c5f920b3049de8fd5dde696a25a6f5f8895ae0f644de52cc553f330f00f65db2967ab815f8c817f24019f30470dfc12ac1bf7b10bddb84bb13ef9270efbe4bb304a3e95ad949aab46c965d824eb670e27592fc78cfe732c56dc38e9e97d4ac7db614dd8012f9ab0dd586a5f7891da9598157c29b1826df3d5ace02a0cfc2b09f8510bd4c0afc6c0bf962d8451947c8f8604f0fa461a5278b8d29928d760927f2b9b355694fc5a0cfc3b59b156e4fc3a0cfc7b0978b3856ae0d763e03fc856dc2982df80d5d13fba5047df8881ffe402f84d182d3fcb76af29d2b21703ff9f6c37b222f8cd18f82fb20d668ae0b760e0bfca2aae7cc5b3ceb031e46f4ec790b76163c8df1d8e216fc79a8d3fbc6836eec052fbd38bd4eec452fbcb8bd4f66196f5b76c766a919a65dd8581ff23db8aa4087e3706feaf2c78dc62c5c3b63070b246bfdaca5f45c9efc5da5b2d2c6b6fb72ab5b7f76192872592bfa028f9fd18b82e01dfabc8f90318789204bc5811fc410cbc9604bc41811af8431878b204fc4945c91fc6c06b4bc0d314257f0403af23012f53047f1403af2b013fa008fe18065e4f029e57a806fe38065e5f568814c19fc0c01b48c0db2e51037f12036f28a34511fc290cbc91047c9b22f8d3d860b771388aa72bd3253fdb3358ea4dc231bc7c6ef8d9f663a9370dc75853388dbe5d4acf2e55f7713d8ba57b5038867f0f4f3fbe89e5e730d33a58625a3b97aa99d6f31878330978b3656ae02f60e0cd25e07315c15fc4c05b48c0872b82bf84e57ecb68369fe552897b194bbd952cf556c0f64aab0e042012c0ddb4f14bf30a264d6b99344d911238bd2a7c9f6a497c154bbf8d2cfd36314aa2558ef84ae46b98e91d226b498a14cf82c29c086dc3d59d08ff1affa83811de90819383a6da85abafa10a63877737c0bdc96f6ac839478786ab9f73d44af19ca3b7304b382c8c46ba28120e7752c9e7b7b1f45264e9918565e6bc9afa4cda3b9845b597f5648bd52cea5d4c8d0e6167e1bedfc362e5750c23b1f23a85e38995f73ec64e67d9d84a919d0f3025ba604a748d4b890f31255265595ca2a6c4479812dd3025bac7a5c4c798291deed0943e9101772647358425ae9d2ed2236265baac8872000fa64bcf3072f4cf601a9bb898b6a743421d421d5979ef101aaa76dc0e660abd24a6f0a8a2297c8ea9d5db61161dc0b228cdb32cfa42b65eb95fa2c370c393e6e20bc39d2e2985bdfbcbc370db492dc3c04baec59ae75ee49fda48f0edccb03cf8b69d54b0b5cd5f62e69b2531dfad2bd5ccf72bcc7cfbc84a65128d2553403351e948212c95be584f62aa613a53d8116bea479b7d83a5d72f8c1c0b3538d41bd4366af5cbb7586afdc3c819e3732c3d975ec635835d33e9d32ce3fff30c135191e23bac233b40d29125bbec94ce27c2c0074ac05b294eb5fd80810f9275c115c17fc4c0074bc01b2882ff84810f9149de58f19c211938b91d1a26e7e559c14fdf3d3077d055eb2baa25502bca0143e83916d17c3da4825d623c5c40234491f1779162649a5f64e9122286cb7a4fe4f8841161700ec2f6d5a1d03ee3bf0f8cffeaac91357139f880e9574ce79158cde0e45c97dfb0d4b2c3c8b219187a3fbe347fc7d2cc092367daa44a6adb05e0b4975ea161f4df6e6a871361b93c0acbe5d124976bb15cdeb136142a36fefbd9c8e1b6c63567ad624eff8995cd3192b2594bb1e0ff85b5c963256df2f5ebd5dae4bf31c9c7859daf6bf80793fc0889e4cbb6aa49fe2f369f385e3a9fb8255b653e3114460c7b42180992481ad981463f338335b591bf33aafe5631692d8cac22981876b68a208c014f7208ac4b807b0f203de15535121c31d7c05b46710aabad0df4e25cf3c976f85b15e5d42359c6907a7a8a1de0c6514e3c0a236b2ba6863d585b911c46cae53469b99414fa28e5b236069eeb02781d0c3ccf05f0ba18f87417c0eb61e0335c00af8f81cf7401bc41185986352bec70195643ac381de9b03835c2449eed54e4c632641253758e1d916f8f72880e26f25ca72237c5449ee750e4833091e73b15f9600c798153e46658d3bad061d3da1ceb8ee43b74fcb5c0b858e4948b9698c88b1d8adc0a13b9c0a9c8adb18aaed0858aae4d18e9682f71a1a37d88aceb359074bd7213e84c8507e4aa3b529786c949ead579c0433d2ca30ed096d5348c2507e6fc6c2bcb22c2625134d1c46c22526147fd6006b6dc05033b1433b0152e18d8611835c52e5193822550e25202ed31ee57bac07d078cfb5217b8ef885153e612359db00456b9944067ac2d2877d81674910113e761851de056518efcc11a994aa78d4c6a18d961bc3aec6c877137acbfbbc6617fb73b66dc6b5d30eec3b11c5ce730077b6074ac7748474f2c033738ccc05e18151b1d52d11ba3e2288754a461546c7248453a26f166871267603df42d0e7be89998c45b1d4a9c85396f8ef6c279d3074bed182f52eb8b9176ac43d2fa616a1ce7851afdb16270bcc3623000033ec121f0400cf84487c083b0f27592c3f23518033ed921f0100cf81487c04365c0c4664fb5039c12e5e0182cf3b639ccbce158a7e734a79d9e1198c8a73b1479a46ce03b880c7c97d5c89cc3183aa55a60dc9bc7a9b87b1cd31961bbc7316da743e31e518e63722a293688cec6b2fa4c87599d230326597d962d607c74320a93f86c87128f9619e7e09a3b2d6c82a1c57a8a3391cefbaf70d93c77481ad7de4388bed1ce05732e552bb3c596168473c2b1ce25739e3e5614c6c8f27f28e163490d4d882ea3453ed604a87ace9f6bbb623a8fe647f728f9e1444a2c27c6622ddb4ea72ddb38591e0fabb93c868b3fddcde3f36de7f10531f3d88994581e1f810d9f2f74387c1e2f0326c5f8223bc0e7e10dcf041930b19d8bed005f80034fc4fc6d9738f4b74dc2cad1a54ecbd1646cc4b6cb8b11db144c8fcb9cea311543bedc29f2340cf90aa7c8b918f26ea7c87918f2954e91a763c85739459e81215fed14792686bcc729f22c0cf91aa7c84762c8d73a459e8d4dc85ce7c284cc1cac06bcde610d38177317dce0d05d300ff385dde8d017365fd643194e7a282535d24399c6dafd3c3a0025913297d056bec4e5feca4d7678fb21cad95c32de4610de0a6a94377759da2b1bb38d245aa646e9bfc52b8b7cd798b4137933ed447a2104d6815c8815ed5b1c16ed7caca6bbd5859a6e11566fdce6b0de588cf5c86ef7a24756204bed3072f08f2cb5436def409c11e53835ac69bbd369d3b60443dee7147929962977799129cbb0d4eef622b5222cb57bbc486d3996dabd5ea4b602ab02ee73a10a28c654b9df0b554a30551e7041959518f8832e809762e00fb9005e260327ec3e2ccb84f631f89f15e5ec352ca1475c4ea81c4be8519713aac0ecf7312fecb7124bed712f525b8da5f68417a9adc1527bd28bd4d662a93de5456aebb0d49ef622b5f5585df18c0b75c506ac7fb0df69ff602326f6b32e887d14d6d57cce6157731306fcbc43e0cd18f00b0e81b760ddd5173debae6ec5927cc9b3248fc6927cd9b3248fc1161bbee2c262c363b12ae4552faa90e330e37bcda1f11d8f71f4ba0b1c9d80d54d6f38ad9b4e9421772207f7d8e163499483c930a2df7248f4c918f0db0e814fc180df71087c2a967bef3acdbd6d18f27b4e914fc3c878df2119a76306f78143833b03e3e243a75c6cc7903f728a7c26c6f2c70e593e0b9bc5fcc4e12ce6d918f0a70e817760547ce6908a7330893f7728f1b918f00187c0e78591437cbe083b3bc46727c6f1970e393e1f2b215f392d211760cdead72e34ab1762cee56f1c3a972fc2f8f8d6291f1763c8df3945be0443fede29f2a518f20f4e91776119f8a3c30cbc0c33bb9f5c30bbcb31f09f5d00bf0203ff9f0be0bbb19cfcc5694e5e8921ffea14f9aa301221f7373b3612253aeed532601275e6773bc0dda21c828571f187532eaec13c117fbae089b8161bc8fde5c540ee3a2cb5bfbd48ed7a2cb57fbc48ed06cc00fe756a0037623d8090eeac077013461039aad87582f6623e90b0ee950fe4666c25a3ae3b5bc9780b069ce410f856cc8a6ae94e0fd3c216e727ebce16e7df8e7151db21177760c0751c02df89915cd729c9fb30e47a4e91efc25a81fabaf356e06e0cbc810be0f760b54c432f6a997b31551ab9a0ca7d58f636769abdf7632435f182a40730929aba40d2831849073925e9210cf960a7c80f63ab1c9be9ce56393e8289dcdca9c88f62c82d9c223f1646824db6d4ab079b945a47b4a3b130a65b3964fa090cb8b543e02731e0360e819fc272f010a739f83436c46eab3b1b623f8389dccea9c8fb31910f7528f2b398c8873915f9394ce41487223f8f3929dbebce9c942f605c7470cac58b187247a7c82f61c89d9c22bf8c2177768afc0a86dcc529f2ab187257a7c8af61c8a94e915fc790bb39457e0343eeee14f94d0cf970a7c86f61c83d9c22bf8d21f7748afc0e86dccb29f2bb32647212476f3b55dd9828c72e6122a73915f97d0c39dd29f20718728653e40fb13e4ca6c33ecc4798e73a4b77eeb9fe186b60fb386c603fc180fb3a04fe14a3a39f0b747c86f943fa3bf4877c8e493dc005a90f607ea7810efd4e5f6043e6415e0c99bf94a5d6879c4862478dadf889075fc980c9006a881de0fa518eefc1cc65a84373f906aba68639ada6bec50c71b80b86f81d26f608a7627f8fd9c64887b6f103c647b60b7cfc88499de350ea9f30a947b920f5cf18f86817c0ff87f9c5c6b8e017fb05031feb02f8af18f83817c07fc36ada23bca8697fc75419ef822a7f60e0135c00ff13039fe802f85f58264cf22213fec62acbc94e2bcb7fb0023cc58502fc2f26f654a76293b956696d39cd616da9e9081fb92ef011c6a4ce7328b58e493ddd05a99330f0192e80d7d291beff4c877dff644cea592e485d5bc7cee070b806a08e8e2c8899ad3b5b105317e3798e439eeb613ccf7581e7fa18f83c17c01b48c07b6793f516cb6b2410c20cb6ef9fececcfa487bdbb1d1461bea46dea9d4334ee15251e811b72d1507691ff21b11116e8243682d7b26071121aea4893bed08b26bd1166daf92e987663193831ed453255722c79d34aae0ac91bec141e8cb8c55e10d7146b440b1c36a207e9486fa5d0696fe5600c798953e46658b5bed461b5de1c035ee610b80566f7452ed87d4b8ce8e54e896e8521af708adc1a432e768adc06432e718a7c0896892b5dc8c4b698d8a54ec56e87895de682d8876262af722af661b28eca28d26ccff5b4a332c340cba66fe484c61b22921388f2e989b691bff28c2f56d050cba41936bfe0ef47ef8c94cb3a23a3895629f499b3b49b52e4c8f9ca5897a3827639dc4811eb56a4604657e982d1b5c7c057bb00de01b3e8354e2dbaa30c9958f45a598f811885243fc5f42af01e4a274c93754e35e98c65c07a1732a00b06bec105f0ae18271b9d7292aa23734c47399c63ea8675343779d1d1ec8ea5b6d98bd40ec7f27a8b0b79dd0303dfea02784fcc908e766a48bd30433ac6a121f5c6b2f6582fb2360deb4b1fe7b02f9d8e011fef1038434716289ce07081422646fc895e109fa523330027b93003d0072b5327bb50a6faeac8d175a7e8ee1c5dd70f4be0549712e88fb948b73974910ec05ca4a73974910ec44ad3e90e4bd320cc0ccf70c10c076366b8dd05331c8259c9992e59c9502c81b35c4a60186686673b34c3e158a6ee7021534760997a8e0b993a12e3fc5c9738cfc61238cfa50472b01ec74ea73d8e5198b99cefd05c4663397a810b393a0603bfd005f0b198bff52287fed6713a12a3e362dd598c8e23b04cbcc461268ec7ba45973aec164dd0912d06bb1c6e3198889594cb9c96944958337cb9c3667832967b5738ccbd293237da989a0b7c3e917eb9cce539beddbadde36caed46345228f5742cc153615cbdaab1c66ed34cc18af76688cb9589f708fc33e611ed67db8c685eec374accabed661953d0393fa3a17a49e89815fef02f82c6ce079831703cf23b1d46ef422b5d958257f93d34a7e0ea6c75e2ff4988ba576b317a9cdc352bbc58bd4e66345f25687457201566a6e73a1d42cc4c06f77013c1fcb803bbcc88045581ff94e17fac88b31f07d2e801760a6739743d329c4ea8dbb9dd61b4bb0acbdc78bac5d8aa576af17a92dc3f2fa3e17f2ba0803bfdf05f0e5587e3fe034bf57608efa071d3aea8b313e1e72818f12ac603decb060adc4887ec429d1a598a93fea85a99761ec3fe602fbab30921e774a5239668d4f38b4c60a8c8f275de0a312cbdaa7bcc8dad5d850ec698743b1351847cfb8c0d15a0c7cbf0be0eb30837cd6a941aed79180cdcfe9ce02366fc0447edea9c81b31915f7028f251322fcc58e2855951235e188e594aef496a73987fc53d9fcc8bb67d322f519f4cef283e1977e4c53c349bb002f6b20b056c3366adaf38b5d62d32647238faab76acf5f62887536022bfe654e4a375643ff0eb0ef7031f8389fc8653918fc5dcdf6f3a747f1f87b9bfdf72e8fe3e1e93f86d87129f8049fc8e43894fc4247ed7a1c4276112bfe750e293317b7bdfa9bd9d82217fe014f954ac9afbd0856a6e1b06fe910be0a761e01fbb007e3a06fe890be06760bea44f5df0256dc724ffcc05c9cfc424ffdc05c9cfc2243fe082e46763927fe182e43bb001cb975e0c58cec12a83af9c5606e76239f0b50b39701e06fe8d0be03bb1ecfdd685ec3d1f93fc3b1724bf0003ffde05f00b315a7e7081968b30c97f7441f28b31c97f7241f24b64e064e4f5b39d1ec04bf8eafa4b31e0ff3904de250326b7bf183f1c62772151ad2867546099f9ab0b99793906fe9b0be05760d5e1ef4eabc3ddd8c0e40f8703932b75e414893f7567a7485c8501ffe510f86a8ce4bf9d92bc077351fee3d045790d26f2bf4e45be162b8ca124770ae375d8205e4b723688bf1ee3249ce490931b3064dd29f28d3a12f53929c979d4e79b30b16b39157b2f869cec14f9668c90da2e10720b26761da762df8a350275939c3702b761e0f55c00bf1d03afef02f81d18780317c0efc472b3a1d3dcdc8789ddc805b1efc23a7f8d935c38530293bc890b92df832df86e9ae4ce82ef7bb1040e722981fb309339d8a9c9dc8f11dfcc05e21fc0c09bbb00fe2006dec205f08730f0962e803f8c95a4562e94a44730c95bbb20f9a398a1b771c9d01fc31238c4a5041ec7e869eb023d4f60e0ed5c007f12b39a435db09aa730c90f7341f2a7b14c4d7129539fc11268ef5202fb317a3ab840cfb318784717c09fc3c03bb900fe3c66929d5d30c91730c9bbb820f98b98c57475c9625ec2124875298197317abab940cf2b587fa3bbd3fec6abd8c29ec3939c2dec790de3a3870b7cbc8ef1d1d3291f6f607cf472c8c79b18706f87c06f615ca439e5e26d0c39dd29f23b987164b8601cef62b560a60bb5e07b98e4592e48fe3e5649f571a992fa004ba0af4b097c88d1d3cf057a3ec2ecb1bf537bfc182b9c031c16ce4f303e06bac0c7a7181f839cf2f119c6c760877c7c8e010f71087c00e362a8532ebec0aa93612e54275f62f631dc05fbf80a937c840b927f8d493ed205c9bfc124cf7641f26f31c9735c90fc3b6c41d628e3edae02b836105db9597ddd66afd002e3dfa5c65d250d6c566ec0ac0c558496d0f59a25f45739565eb5775556827d8f69345aa65117658dc6508daaff4a7ecb5392f4074cd2313249074795b4d8f8b7285440570c9b725630e68974f9a145745d6c01fd05aee225ab884be96f8471f1d719c66f045545ab1f31adc6cab41a806a55dd0aac16556c3cad30fe5d6f48bc98021646b1296b6465157d7ec2f41927d307df6f6e95c0aacbba6acce7d2af969aab9915a4fd1993f60899b4438435e930d511869c2407d633597399ac5639cd18d599462ab8fc6e58d5ff30bdc6bb970beb5dcb855f306927b827ed06d7a4fd159376a24cda7ea8b4136918d165467935259d489f47933262636ee9f21ba6cb24992ec31474c98f514aeddb7fbcbafd8ee93659a65b16aa5baed16e2da32d05d676701b23795360fcb6da90dd2d2dfec0b49822d3625c941aca94b5d4d086e40fd9f941f6d294516d16d3fc2ba2ed73cde6da9f98be5365fa66d8b6c811469b5e1143377734f80bd3601ad64784a183471a3f91bec8749a1fc554b322b69f894838cef8cbfcbc48d23f996c68b99cd582d1fb2e2a1afd8d6994ab564ba8f611496b996de421e1a18096bf95367a646abdca7f30ddf264ba8d72b5ff0bf5c863a18cf13e59750c153dffc5f49c2ed33347a847a05dc17a64546895f1413eb3d3f531ecd17b4b25abcfa45ace90693914297b93689eada5fd8815ac16b0d6f4138cdfcc5ef414e33fd22293d680bc21c3339fc13747d07d8b2a9a69986633b17a514512ab7665e0f771925a26d28b55d1208c6930cb3d0d48ed389dbe574ea5246fbb675d3aa6c191987559db1332aa87d6349a8dd4f2256500be6dbfe4c0af54344bc2349b2dd36c045a0b8e315e22bb73abd717a3e91b0531f45ca080aba25f2d4cbf39eabe83687e9ba51239bdd32a19d36aae9a5678ae99a3d811f44951688da497e4be47a436a6d53c995683e2d02af15eb63a984ef3653af54575aaae0bf75bad64dac62a5f4e73a72ea6c902b55115e6afe21657416b8c722a2be9892fa27d0cb7b4a88769b1101b55a9f513cc5e9fa9f92a0a3209e48f7d7b75ab7eaf8fe99b8f7919a3b75ca49d9d44c790c586bce5861ea456ac947a2670fb8bbf256e80e9b348a64fd798fa98f93596e6eb325acb1748ca527cfd9e8698ac8bb11203d3cca66dcd1adaa3e1659e1c3c9363fcb79af9a95546b5f03b152d1a615a14c8b44817b4e0a3712e7f398de5617a1da2cbe896fc8d31f90bb1dea79d5c80fe5cfb3ac467474d300d96c834989590dea7f3b15ebc355a538c8da53236c6d7301beee87c10a6f33299ced3ab5aade9466d564a7d29a4f75a42ff366d90d8f312e3133362ceb8aa3747515b276f4ca33a14b2de5f3968a5496f3e9bfa3a2b296ee41baffb2007633c14c97898ed210f66bb318546195a4a7b34253469a26d3e5a3be36cc43b12688631b25cc648a1c796e18c11bba5281ecb698ef1b4c28e7f285332feaba8f248102ffa0ae3c332a1671ee1339f8da3c8d779c693752ed60d2d30cd8ab1118673cd4c4f8bdb9ab4c43429717f46dcf4e1793b23de0ad36725566bdb9fdd94cde6acb7ad45f45ea3db3cb4c6782855f3b463f95ae142beba51c7b4c1f42c73d2df8fd6a78fded7886f16eb104c8b55def497a3e9105f7fb92da641b94c83e18206913461499b60fcb6847a24c8acd41285151eb8e6f169d70ed3ae426d251aee258b942738bf614fa778cacda1984695328db2518da6b1bea939632faeef2a61634eb33f521ca566982879d3a98e87613aae96e9981963763b229bb565233e8c7cc600ae5f2e784b4587144c8735321d72157c99f89c626c047befb8e57f6a8f71b056c64177050e962af8735524ee8049bcce4eab547dbc6cd6f0e418e422da1627c6efd711d362bd4c8bb18216d674adf57a29cd13b3bf305952d789f31cf6e740e2d7b613a6ed0699b663502b9b60ab665c4b6b33e2a5135707c9ea7eb7572c76c674dd88f973ecd58ce648b48cf544d61bb961f60f5730ed79df6f01fbd729aa8ace5d309d8f92e99ca7d8e241df844c5237b47523e7bb622c6c92b130cdb59c373d15a3e95335af9fdbb69f8a31b0d90d3bc80d881d74c358d882d5edd1c68764cd989febbbee98b65bd5e65155ad3e9bb544e5ae687138a6c5d1322d26386ca116d37694d4d663aad6da463cced8f8a6fa9b4eb5ee81697d8c4ceb898e2dd52f7af7c4f43e56cd8333a16a443991ceb8aeb0f4c5c6513fa4b92ab928aa8ed571dc195bf7c2f43c4ea6e78c383c76f6c738a27f2a1e2f56fc3dd0de1813c7db59f186f7b7a7d3f14db9f12e5943b134caea73affad669986627609ac99927165666bc645def3bdef8ff7ae38bb228396bbe83ad35885fb3744cb313659af5b0ad191f991279d633e9dd933a0393fa24cc5ba79e1f644e7e2db3a2c4e64926a6ddc96a2bf5a2699778adb230ad4e9169754282e7a3adabb3acb3d36a72604879b40ee3e5c2cb19ef3e18d3a762be513bbb8046185214b01d2635bff3a72fa6e336a77bd18a6a5cb77e986ea7c9741b6d33ff72e90aa34a6166b96635ed8f697aba5a4d17bd479ce85dda0330adce501bad4d63fdb45873d3e531d78e4f8b6395c8404c8bed322d7ac5d4429c75280a6db03dd7aad6231f84497e66fc73e8d699bd1cdaa214512d4aa3f611ac184ee7d0d578188cf17056fc3ba5333cdbd73d0493f66c27ad962ca76aaebe1b8ae9b843a6e3b238d7c3c3fd66a554735edb8fa0b5e434e3ef4a6114e5643c39958e4ecd7ed04a84e778ead161185be7a8ed9d8f557f96092c79b19e6538a6cbb96a33f0b1db02bb792bcb35158d46601a9d27d368719cb61c6c0b1e8971b453c651655cbb18cd515d1eebd7f19d2539549f7ce55da6180edc71ec069e0a8bd9188be7639e4437244c7c1d9183e9790136a7e1be9e89ab3f4661da5e28d3361fad3f6472c4b7de515e0bd8f5a1ba6d0da3317e2e92f1333f0ebf323e2274d7bbec36336330662e569b5191319343eddf5b0f7bbcbdc7b198de97c8f49e138745c87c3cfeb68571182797aaed9595fb4a4784d6d1b183ca7aa57873f7084c935dce476b15aec7421b8f497b997bd2ba37b69c80497bb94cda9e51faaafc3c71d86612bf5331eb251521ebf0e2937b2226f7156e793264ad233e0ab6e7e370df933109e36177fc355f860b355f460d723219e3e4caf85bc10c8579d79ad27b0aa6f7556a2b8a657a4722d1f93befa7621c5ced06074581b0ff6918077bd45680e1f66f7f6d493cdaaaf57372316daf91697b84623cdb6c16f1b590d6fb53a8a763a5523fd87d8df3308daf555be929efe517514df3695c81429ff6f7a7630c5ca7e62d963130d1b85b437b34649503a9e50b583fd74d1e12b9637006c6d6f56a3b8262ed188cd713e4863f7126a6e30dd81a7f3b732463e80c22f77ce4d19d3e6ecd94c4bf5e6416a6eb8d6afe2199f54fa09a9605dade8fc4f8b949c64f491cfeb3d82ca978cce4efaa33e6d4ef381be36daf8cb75d286f538c72b29ed6048babe2ad4e66eb8c88efb5cc120b78129bbd28662bacddd8756daf858771a0b1fd9838d385b4ff9f47bd1d4be988b71c8951e0ce6ada3958eedc8cc516b4b70f9744b3c76a36f55db8f1d76a7331fd6eb1b3531aee001463c88f891943de9de821f3300d6e757b56369f45f1adf42cd2de7c4c97dbd4d6b8c413ab788aa09f5a24e20598e4b7cb24ef1fe75e0a6cf62b9bc6ab5ae9daeefb85983677d889372c2ff5f67a35892cfbf9989677da393f225ad9573b3fc2ed38398b30bdf6616b3163e75eacd5a6898a9cb018d3ed2eb558b7d8fe75fb3e066ff6a61760faddad16413ffa590745b4f5857d08f73428c434b847ad0e8f1d01d64e6c0b15c9976092dfeba4af03f7a68f9046514e54d9598ae9775ffcfac58a219fc8fa7c19a6dffd32fd3ac48c92cf57f24456dac862feab48588449f8809d1d593c4515fe23d263e715b8c3fd724cb3075563a314564571c5ebafc555118ad6b9a8c30a4c8787ecf47a306f4eacd14ea2fd38c598960fabcd14cbe75e2b5d9e712dc1a47dc44ea450959ea819099997a69aacc756623a3f6a274e8abd31b77fb42dc5b47dccce087c3a90c9e9087c7a9ced6a19a6c1e36a6b89abb740396c0688ac025c1f675f349e11c42a4ca327b09a502dc23a89403e99eee68c1ee5ca4e54f5f82daf1cd3f2499996472a6b698e02ad7aaaac1075c24a3cb95e81f1f114b62a27fe28a4392cd2851b725762723f8dedefb22b376498484966274df9a7d192497c626b5d5a9bbe1ad3e219b59ddfb1e252b81b43640d26f5fe78e2bde25227caf3b516d3e759ace6b3336b20b64c306e15de36a921ab68b90ed3f2397763a08c91ee19a988433b7772773da6f7f3986f02a629e622d1d99c5faa90f47871d9e329671b30c95fb0b31b4f943c9bd6bdeba3ca6ce7b9bb314437623abe888d219dc6dec5e58f77bc7214a6c34bcee34563b586dd11642267c337613cbcec666c29bfd52f9b31ad5fc14ef9b023e538308365a7e5501fc9d99742858d2d181bafcad89814071b4ed60572242fac7f2ba6f96bd80955f1c45130230597d3280a136b3432f2d198beafdb699de41e8b71545aff78978fc1747c03eb9f5b47197c7f1fbf4e606b1eadfd5bb7fbb3c76252bf89ad0e50917a1a4534572e8e50f2b5c46365c761babc6567df69755d48ef60052d61b16a135c27bb29a8e8793ca6e7dbd8198a2a7966cee4aac8ee46de9d80e9f48e4ca791b66a087b11741235bb7622a6e1bb320d972be65a1e5df765f6772230d17495a3477235320764d70ea6d1f313dcab514fc2187b4fc6d8e1cae579a9e50c1f7772f9644ce6f76532778e310e8b9c485d485b6812f9ca1d394fc1e4fc40cd8f612fb66b3c5e19376a9553312d3f548b3b3ec1560c66f74719f1e8bc0dd3f92399cedd62ceb4aa455237bf5191f7344cde8fd54e04b6b7322efeb8d02a3a9d8ee9f4899aff2cba4e647e6005603d76be781d49ed0c4cef4fd56208479fe5978f6fec73e05ccfed989e9fa9d52baaeb502ba2acdf58603b8df87cc267623a7f2ed379aa87f1407959367785f14881d557b9af71755dc45998fe07ecacfea8ee61c56659ecf852bd6b31cfc6b4fc02eb87cbad8ecc4c2f325e5a5ab537a534c68998899abbde8169f8a59aef45aee14c9aa72b6d9c009a287dcfc1f4fd4aed9c19b9bea3a8b5ca3c8e35a1ebb998ae5fdb8b49e8feb9c178df2edeb39ecec374fc26fed5dbb1a22e266a8cbc13d3eddbf84ff5337d50e6495605314fbaf352bbf331edbe73ebacd0f8e29c267a27e405180fdfabb531b12211c6b71fc98d7ae8424cc31fe25f493d8daec65fe68bf3292fc2f4fb51a6dfdc1a3ea31b5bcd1c2d2a543cabb72fc658f9c94e1c707cc749ec798ef84eaa8fbf2f7c09a6e9cfcecf8fed1bca8ae3fc58f2958a0697621afccf7e092569aaef7530f5c39ebb9543bb30fd7e51ebe5465fa7147b1f5ba2222d5c86e9fbabda892fd1f5b5b362d8edd6f2724cb3df9c9c741ceb44cc44b5245760dafd8ec5c9315b9271b4cd3623c254d2babeb26aaf81fc37f555c4f6df56d17837a6f11f6a3bf862edc75e4c7b09c5f4dc1477e78cafc434f8d39dfe1bf77e71f9e3296fcef6995f8569f8573ca715da39cd7e4a0d46a9b91ad3f66f356fa67d6d4751dfbbb9929d8f342a12aaf31e4ce77fd4e27389e7d164d355fa259295ebce5660392fb5d7601affab76764bac7ac7b94dab68752da655a896d2b942f6b57266bb2aba5d87e9a6c9749b1c63af7769d5984935ce76e4fb78fc0491af5574bf1ed33d2cd3bd8b6ddd2371d3dd9aebbb019354afa514d943ac49a6b156bb008c8d54cead8daf96b811d326c91ddedd8bbf7c1326692d77245d1487a4b58d7f35adbef1e728ca3e59931389ce5811ea4d065a5a5a552fb657959fa397c52712794ed22f31642132e65579347a84526220241bd2258743e63f5d354260257369684b843142afaa79d642b04e8ac4c81a437d2fe65a50b2b389af395a48223f19ffc6d242e6dbe905bc3b50bbd83acd33fe256f9518288b698e9191f622c30a6a338d3896c13469b2ff739ad626b9da81e56ab837e98a81754f6ad6145b8a3a24355d6e432dab310bbd6932d94d80496cf4a435346e48014bb31cf1c8976ac54c0d8ebe64a9f5a400dc8b17130e6eed95c1b5a626662e8d2c675ba5f74e081919124e23b950d74baaea19373032cb581abfbc2010242d1c1a21a99e9724d5a5f6349d5a5129dd3e4ab65e0681a209c32314d54f941d911dd6d3e9c8ac3c102415eb11921a784952236a47a534e05b6453b2491539f2390864656545c86ae82559752859d6bdce4120683ba8951a794950322528b2d82708e4ec191621a7b197e43401f5118fb4bad6202b9ffe3d857609824018f98713d624717da629ec78b020f599f68c8950d5d44baaea53aa8a99ef9f54dc05ecef20d054a75d84a683bcef3589ad5d10287ab06784a283bda4a831a5c8dcc140ce6a0da645edea14a1ab99f7fd27480f113658b5d46f4d226435f7dab622914b0ae96a98028bc738087491c5959cae165ed2d5c0d2fe65d360eda4ce1a1790c170418b08512dbd24aa36256a75c0eaa847013dadbca4a785c58e46515f4105dda2bc884e0215b0cd1205acee0a0279474d8990d73a71e4992b80cce1710e9db1303dd4d96c877430c8dbde37425e1b2fc96b29216f143db6cda4ada0aa83c1c3890581beef017d8724a6bb0a490b02455bfb45286aebb593cfa4c81a1c3208242deb1f21a99dd77558368b74ba9416b64a56ecc85c6ba441201d8ba058d8d84111f20e4d9485c10d784120e967500c0ff3dab785b99173a89f2b18bead777a47084bf17e98cd5bc6e9f478e7e0f9b7764d8ad0d5defb9690d3358ead6c080245ebea4428eae0bd27228f6d928f84d9ada00b4b83e1e5da3924425647ef8b5f298dfb02cffd1855b5ae210874e5758bd0d5c9eb7ebc9d0ec474ba0092bc1388c9fac111fa3a7befc889d06416cee00cb69b8029fb2e5e12d52caa9d4da9721906a5fff51be8e177f592b856a8978244052ba7bb27cbd94a2d72b66e4140084c011dd854ef5d88662fdf0cc51e88fe7daf083dddbca4a7b98d8219accabfc940425ded5a02f868d79650aaf18d2ff66c4c97d7aea35372e514af405cfc49e205fb4ef2ee70f166aaadd50fa9e89aad541b7347a9c8282135e6245d6ad4a99654c9fc426a9419f7541b73a8a9367aa0a9c87475aaa4b24ab535024f8db9dc2b55d9df9d1a87933735aef62a15f187a6a22e9a54d43d98aad4d54855acff52953d6ea97174b153630ef952d1b5b4a9d2b56ca9d51670a546edbba622cb5053632cdd498db11c3ad5867b2455795a2bb25b41becb208dec5638fcfff38d04696423410f42c277ac2f61f62e7a7aef3d5c60b4df7de8d5dc3fb2d278d93c6599bc26a4d9b65a9ad3d8a65ff3c035de7790a55ed7d2bb186c47a1787a18776a91ce59af444c512e66f6571975db547fefa86ce715952f832587bdbda4b22da332628176f639923dab19ded13acb2b5acbc3115ad312313c85b4460bcdd9df50c3333a5b7845e76cb0683fdd4f740e08249d27033a33fc44677a20e99c060a7ba69fe8cc08249d1b009d597ea2333390745e0ae8ece3273ab30249e74380cebe7ea2b34f20e97c05d0d9cf4f74f60d249def033afbfb89ce7e81a4b335e8280d48c4383d9d45254ae8383dd32bfa268271fac04458e352d41ea75bc2200d308ab76774d6f58ace0580ce417ea2b37f20e99c0ce81cec273afb0492ce43019d43fc44675620e9ac0fe81cea273afb0592ce3e80ce617ea2b36f20e94c05740ef7139de981a47307f0c28ff0139d9981a4f35d40e7483fd19911483aef0674667b4967574667b1f152319d762ba62b3dccf50591f87776a3bba605706e731d18c1e72462072166b9d6b9cd0c2fc9f46c76f31740e6283f91991e40327f05648ef61399190124f37740e6183f9199194032ff00648ef51399590124f34f40e6383f91d9278064660107f2117e22b36f00c91c04c81cef2732fb0590cce180cc097e223388abe866003227fa89cc010124732620735222567762645afd4a7d023801d7085039d92f54660692ca8b009553fc42654620a9842198a7fa85ca3e81a4f20540e534bf509915482a1f0054e67abde333daa602788aa3874eb75a5e11f933981bcaf392c854c15d5c5ab54d4bdcc0f85f76189702bb9dee25dded05bacb8453b9e0e980788315bc8a6134207886df094e0b20c1df028fd34cbf139c1e4082ff0104cff23e525259d5441da98ba7193f1618f7ab8d4f8337186d02e63e8ff43a924dbebdb30b02b070b63ea06d7622a78c8be886e471ecb0bb1594be056c176f6e282734de50836f74e657f8a687f56785ee11d91d410335a726070110ced39ab2b65776db19d49473bda4b2356ab7ffadced322609bf3fc46686600092d0284cef71ba141ec7d7e0a8afc02bf111ac4de660f60a10bbdee322da6416dca69eb5d0942bb048fb654d065ca4f4480dcbe89df69d4ca2bf24e02e42df292bc0e48215e498ffbfe2ffb993241b15e5c93739c305a16a9213d9c305e16f688cc170199057e22734000c97c099059e82732b30248e6a380cc257e22b34f00c97c0c90b9d44f64f60d20994f033297f989cc7e0124f3194066919fc84c0b2099230199cbfd44667a00c9cc0664aef01399190124f31e4066b19fc8cc0c2099f702324bbc24b3bb30c85ccd828f9aa147cb58f4d5683be1646f7a58b3deeed5a0730ba07c65229c22fd13ef14f16c32e377e01429f5fa7089682b9ab8a69efa33cbbda2712ea0b12c1175281e6d164eaf791ad6c2339bfc1790b9ca4f64f60b209961b0d2aedc4f64660590cc3f816556f889cc3e0124f36f4066a59fc8cc0820993f023257fb89cccc0092f92b20738d9fc84c0b20995f0032d7fa89cc20ae3dfa0a90b9ce4f64f60f20997541d768bd9fc81c104032eb013237d4ec0243b8bd3588116992c16aa38d89586da412382d2d80c1a9fe01b67994d71e227165bb79b8522115a6dc3bf23a7b455e47d0e46caac9820d8f2df4b4606ff76cf91628d89bbda4921c2456c18e7bf3d4ea3c5bacd51058dd16afcf363717684da1892fa1279279465843af084b02846d4dcc71611989f7835fe9157db78126e2e8449c71c5e76d2afed30b026b811aef989a3d850d76b183388eae0405fcd844147012b3e8bf731ee059a0801fe7751fb030ea01acc1dbaad712d8def189a81d613196d78ee38dffd6d339ec023ad0f3703ae15daf68dd07683dc14b5a536c1fac38867d62ed6c7bbae6d7b3363d172c0c38d14b7a0fb54d6f1ead3f89652fa17025de36468d3deb6d8276fda444586ea4bbb48cee152f8c12b123d7f28e87f49ee7d52aa23180de93bda4f760b2ec23ca29e8938deb326f87904bbcb2d1b741ed7a8a7f4e000ea247722aa0f2d44404cb4f8b11aa6741a0fba2c3415f745b4dbad4ac2b2e83b8de3205d494a77949651bcb79f7e6d53c809e7c32d278c56c7ec889f64be91b44c4ca000e343f06d6797a22f649daed3615196f88fe744febd3fa9e4de302ab3dc37f27aa67d29dbfc1ab5797015ab77bbdabbc8c26bbc6a804c82b8ba903b9d4cbf8459ed1d615b4ee677a3f0f5e443670d05ee5621a528738484a8dbff3692ca362e3babeaad50f9eab241d9079969f4e57f5f40046cfe85c09c6f067fb89ce0181a4f30a40e70e3fd1991e483aa78306e71c3fd19911483ab7013acff5139d9981a4730fa0f33c3fd19915483a9f0274eef4139d7d0249e7eb80cef3fd4467df40d2f91da0f3023fd1d92f90741e0a3a4a172662b63d5d88f696100fa76781ca36001fd245898cb5453c43c52c726b1185cd67abe0fe6b3ef98da0c05f5c93d31bd615c2418c46580eacf5929a8c8020cebaf709e022d77250735eea2732b30248e60a40e62e3f91991940327b01322ff3139919415ccb0ec8bcdc4f64a60790cc03a02dbfc24f64067197c56d80ccdd5e92591f6c1db01e1212bc055db5c0b4c595fe399d323d905df37c608157f989cc201e29bf029079b59fc80ce291f22580cc3d7e22338847caaf04645ee327328378a47c2920f35a3f9119c423e5ef02645ee727328378a4fcdd80ccebfd4466108f94bf179079839fc80ce291f20f00326ff41399413c52fe7f80cc9bbc3f00b138f16bde3cf39f6780c1e35ebf9c309d1ec8b3d034e056bbd92f54660692caf30095b7f885ca8c405239025079ab5fa8ec13482af7032a6ff30b955981a4721fa0f276bf50d93790547e09a8bcc34b2a9b4bb6514d317e245b01f859bb09d83cb555f388c8c16029c19d8959379496f8281d9ed9e13840df3eff1c531cc4808c19a048df958822cd9760adff8f1dfeba0d1079772236ebd90bc738829e699a4fe9f690d61caf6aca5aa0a8dfe325ad87c5b907729cf767a22cf0eab8f706c066eff533b9e90124b73120f7be44c4952131388a68a4b819061bd986d0e6593ffc4a769f9390276be8f283d5745bb4a7ee0fcfc23efe033c49f77bdd685923b54e6451ba080871cb8da0d65a6490eae149d1295e11793020f201ff84630ea297bd3d700c3f98082fbbdd70cc0302188a793db0cb87fc4466ff0092b90690f9b09fc8ec134032cb00998ff889ccac0092590ac87cd44f64f60b20991580ccc7fc4466df0092b90a90f9b89fc84c0f20994580cc27fc4466104fa99805c87cd24f64660690cc6240e6537e22332380642e07643e9d881376fbd09dbc91d9a0044c69783623b40cf8399ff1cff1da195ed6919e857b7c1fb8def6fb89ccf40092f90120f3d99a3cff4c0c199116c0801153401df99c9fc84c0f2099330099cf2762169dcca1afa16edfd5b440073bbe7021a0ef057f1cea9e11c043dd17021a5f4c840f3d8df6230b298d647df0d2aa33a98a42eb021d53f85dd0097a29d14731d89f34eb17c029b33bc0f4c4cb89597134e03fb4e2e8586099aff8fb0c96f4009ec1f231b0ce5713558746e268fd974e0d18062cf5352fa96c272ce82a61872c8d200bf4d8ba0d7cd745f0aa807a601cf4ba5f890de2cab99f41e17fc33fa7fff5096007ea78d0177dd3eba02665f49005527fc28f83475a0f40da5bfe3cb62e3d80c7d6ed05b4beed25ad0da5c72ce4312682b792a83320ee1dff9c4815c47de40f032adf4dcc69a859891ff578e6603b03f425df4bc4781c5aa2391ecf0de518b56166d5ae9f3ce38d15b4e189788d3cecf43ce8d57ccf4640edfbfe3cdba74f00473f4b415ff203ff84bb1d10c06ef99fc0423ff43ad2c16aea8f86d36591963c78f56627d0ec7c948828ebf6d702077195c65c60891ffb89cefe81a47334a0f3133fd1d9279074b601747eea273ab30249a706e8fccc4f74f60b249d6980cecffd4467df40d2d919d079c04f74a60792cea34147e90b3fd19919483a5f01747ee9273a330249e74d80ceaf1231e3137b8c6e9e254d400ad8c1c61eba86977876280d984afbdaafc4a60590d8d381fbe39b442cc1ce4cbcb7d3b3e0f6eb4071ffd64bf20eb16d95d6b51d1e7a912ef06af5f009a0a87fe74752fb0790d4e300a9dffb91d47e0124750220f5073f92da3780a43e0f1aa41ffd486a9f00927a2e20f5273f929a154052cf01a4feec4752330348ead980d4fff991d48c00927a1620f5173f929a1ec4d61f90faab1f494d0b20a9e300a9bff96736be6f0067e37f008ee7dfbd5ec964ae8c175d2213aace1f081e7d29606cff87d7f4998b1972e96ea125f4f37c7ab6f444b69e33780b1a4601fafef492bed6c2daf7c81ae368c1f283678f6560b0f997df080de22682c301a17ffb8dd0206e77590608fdc74b4233d1b59f7c0f2669884ae83698954c91425ab39ab56a850145b6bbaea549ac30eaaa35344b2a0d8e88339a6f95f5d0aa3ff1ec905a50effeeb65261c2cd95b5cc6d6dfad36fe4ac0aee21e5e91381f90184af690c42e42d5806d30b4b7fc363d80cb6fbb834a434bf64d60febe01dc95f404183b859313b261bb6fe227f33c8ba7540cc64b7ab26fa36f073194c07b80daa444586666a87fe22dd3b3bdee1a28d8b5926b300490b58e4c0f642d791e68db93fd4466bf0092790920b3b69fc8cc0a2099670332ebf889cc20ee6b3f079059d74f64660490cc330199f5fc44666600c93c0b9059df4f64a60590cc3300990dfc44667a00c9dc0ec86ce82732fb0790cc4b01998dfc446610cf1cdc05c86c9cec9b53878218e0a21d184c36f192ca643a2d5c610859e1ed14b0677398ad81d5354d4ec00c91cad6ad209e2bf400f0031de437428378ead58380d083fd466810cf17ba1b10dacc6f8406f1f4ab7b00a1cdfd466810cf19ba0f10dac26f8406f114acfb01a12dfd466810cf1bba1910daca6f8406f134ac5b00a1adfd466810cf1dda07086de3374283b8effd2e40e821c9095e75440699638dbf8be98a3acf571d7936dc7c1a0c37db262738623c8c9d38d278b182ae2f22cfa618bff365dc23e8122f4f57c68df1eca808e0f86897ec7174596e874b28b91c9bac8bf78cb85a5e1117027679684d3adf12b66378bb57548e00367858b22ff60ce5d23546090861719e577b86c6035253927d7b064c4600176ef502d4b6f792da7ac64d09a5ca8cea495a7312bcd733ca6679769a06a82d3b7849591be470a7c5f4b831d86a8f4b4cc7c8bb8d0580d28ec9090852259ea3f1dfdaf8d2022cb9ee94eceb9389b202b85ab32ea0b773a29b2339bd7974e92b991c5e42e14abc0d63e159bca574d01c7549cc3ae2f4ff5070fe5bc128bdab97f41d069a267e2536b8de28e8a4099a400798e4ca9bab7174e34b9047efeb01b9a95e929b2634521506ee32dacecb62d2619dfe68df78d8a8adf28a7e1d540dddfc3b081810c041c0d780daee3539feb79e8d9d15c093b13b022a0f4faec1f016d313d58bf5cce53c110c0a7a78496527a1be2d60e7cad8db9539c2f87f391d89794ab367bbe2be0416dbd31fbd5973737719b5ec4276baa187d4deee95053706d4f64ac4b016ab57c593ddfb07f26cf71c30eeeaed273a070492cead80ce343fd1991e483aa780c29eee273a3302496721a033c34f74660692ce8b019d997ea2332b9074de07e8ccf2139d7d0249e74b80ce3e7ea2b36f20e97c07d0d9d74f74f60b249d2d4147a99f9774f6b07d18f45483a342ea6fcda3092ca51384e52014563e1da21651244f474f0dbc1a98be0ec6fffd13e1cc8e6f9ac5d3fd769e4db4d406363dc0cfe4f60f20b93782fa77a09fc9cd0c20b9c7017207f999dc8c0092bb19903bd8cfe4a60790dc4d80dc217e26372d80e41e05c81dea6772fb0590dc9d80dc617e26b76f00c9dd01c81dee677283b8a0e84c40ee083f939b1540724f03e48e4cc4c098cfd016332aa7d1e9c102b6ce055f0d33017ddfc3a6aea167010e01edd9c93e3ad27540202302cc03ebba72fc4467ff40d23916d039ca4f74f609249d6d019da3fd44675620e9d4019d63fc4467bf40d2990ee81ceb273afb0692ce2e80ce717ea2333d90741e0366108ef0139dc10c9df41aa073bc9fe8cc08249d37033a2724227ea4bd9087595e8e893c0b7af82498cf9ae82732d30348e6f380cc497e2233238064be0bc89cec2732330348e61780cc297e22332b80647e05c89cea2732fb0490cc6f0099d3bc24b3aded4542e2e96169013c3dec02d049caadc9ad69d6d150bf004605680f2c34cf2f54660692cacb0195d3fd42654620a9dc04a89ce1172afb0492cad7009533fd42655620a97c085039cbfbae110970984df7eb93c3550b0d41a6189f14552de69d46037815d1e049c15b3fdd18b4e1477a496647619a780125ad8246a028a72192febb1da579c062672722724a9a70d0f58244444ef1acc0df003cef73fc734a6d108fbf9a039627ccf592cac6962034098de0e359312e0175e53cff9e519b15c02827b541119fef25b5cd6284e6e09a7a1ae1b4dc2b1adf04342ef0cfb1577d037956ed4e40e6423f9119c4b36acf0764e6fb89cc209e557b2e2073919fc80ce259b5e7013217fb89cc209e55bb059059e0273283d8593f0d9059e82732837856ed2640e6123f9119c4b36a37033297fa89cc209e557b012073999fc80ce259b59703328bfc73566d1017c7d401bea2e5fe3aa7292b9007b03e056c7385df080de201accf01428bfd46681097bd3e06082df11ba1413c80f57140e84abf111ac403589f008496fa8dd0201ec0fa2420b4cc6f84063134f64380d0557e23348807b03e0c082df71ba1413c80f511406885df080de201ac8f02422b1331abde2ff1b3ea9e05623f19ccaaaff692bc76c872ae09e8b16d70ed51f09676650262d7787d7ca079fadd528342f3e0d5a2d0ba007a8fc603cad6fae7b0d58c001eb69a06bc47eb1251270e487c9de8d98ab73f801dae4fc482c10cbab632c1f4797608e036d01e6f4884ed65588eab0cf66acb2301791bfd53070e08601d380ad48147256266272d6afcad84d8a56791b56601bbdc9488d597f068df227a4222efdce41b1f553fb8cff3d313bb79b62a1834379bbda4b68bb00920b2461d76b5ed6e05480fe01ae239602bc096ff57de7dc04755657f000f20cc4440044550441011c182d38b1d089d0006107b0c291049082601c4755dacd8c5ba2a36545414bbd87bef057befd87b7775fdbf776726f94d0a1976ffe7cef9cd7ef61359c26408dfdc77df7de79e7b8ede4e897ec21ce2ed60b65d64a39c646a14d79bfb7fe60525194bf89e00b8474ae2767137eecd6c5b68feea59a65f6de2999c6f31d50f66d6a3b2d972323db78332b510288f96a474ffeab96947fcf82ed70d00eb18dd5de6a3840f987eb88b1fabb7cb3c6397832170a339cec6e644ea2e5e6a163f739acd9c965af58add828e02d0c5da40fd84a07570f11f9fdd7b3a7b8ae105709b3a414f73733f6173f3cde1323f51573fe338e1e6d96170919fa4ab9f3123e74ae03c59573f6346cebde1623f45573f6346cef380f3545dfd8c19396f03ced374f53366e47c1e3897e8ea67ccc8f906709eaeab9f3123670fb8b39fa1ab9f3123e736c079a68dcc8f60369209674bf17d0a9bc467d9a8f4da76602e170a988560ce3c5b47a7b696c3c971c260f2d3807b8e66dc1821ee9380fb4fcdb84142dc5d01f75ccdb80142dc5d00f73ccdb87e42dc9d01f77ccdb83e42dc00e02ed58ccbb8f57c09e05ea019374288bb14702fd48c1b26c43d08702fd28c1b22c4dd1f702f96c41dd86497bfccf9837ae723b3c7df61ce7f6b4d3e9fe88a77513b21e669109eb944cfde7f9c70efff65d8fb5fa62f54239c9db2b714eb2130115c6a370bda9ff359d0c703ee65ba922c4284c1ee72984d2fd79564c1c87922702ed79564c1c839052ef62b74255930721e079c57ea4ab260e4bc1c38afd29564c1c8f91070aed09564c1c8f932705ead2bc98291f33de0bc4657920523e766b0505a29c9d9c9ed2960ce380b1f2413dbf9df099ed2afb51957aa3671a572f316256b79662f6cf63ac107cbfe52ccbbc1057e9d2473cfb4be5ca95f1b0b62a5fac509973210bbb4af86f17a7d360374e9a79c8284e79c5e855caa1ba4abe8543514611b6b0e7f8f3185da12a1b885727893a5f0a2300e6fb4d157b3ede8a65bb1a4da9c6c9e93bc2561345e70d65cdd4108f962b88fdf64a3be4e66c5fe83929862f1f862c0bc5913a69f10f320c0bc451326e329d1d300739526cc2021e6e98079ab26cc1021e69980799b26cc3021e6b98079bb26cc0821e652c0bc4313669410f342c0bc5313668c10f30ec0bc4b13266312cd83807977360b68342de3cf17eee80c94f768a10c52529e0f94f76aa10c50529602e57d5a28c39494cf02e5fd5a28439494f702e5035a282394945f03e5837aeee07142ca4140f9909e3b3823e52d40f9b09e3b3823e512a07c44cf1d9c91f21da07c54cf1d9c91f251a07c2c9bfbe4d68ebb8a75f998095bbd8fdb38c83233a39086a5cddd0552ac45c0fa84ad94987a53fb26d1f5a330471afa54421ec793f20df6323b77e9629639bf2b4f160816dce45d225572b50092b69ed20aeb23841d0eb04fdb98511b930e132d275a4f364cfdb9645d06a97499c970cf7fc6466d31bfb3cecc9da67cb3611e7d369babcff45e1e3ec25e1e3de1027f4ec7ccd9b44c5b80b250dbbd00fbbc5658c6be5dbf02ec0b5a610384b07f00ec6aadb04142d8ae70b77f512b6c88107643807d492b6c9810766f807d592b6c8410761f807d452b6c9410b604605fd50a1b2384ad07d8d7b4c2c60961cf02d8d725617b18d8b6fb26f3754bee0d81e9372409376e230e90fa978a464f6ba51897c3487c539231dff94de22f9de7bca0def955f418cc0c29b0b130eede92041bd2ac4b778dbb8d602c129b78eb5eeccf4f58ec2f0f46e8db6ce03e42f06f2116f08e8db6aaf579b3cc39e3756b8d1e276c8efe0104afdfb571d36a6dbfbff1a6e523bc694d8239f83d1bcd54db5e9716d9489eb8516aabaf182ef9f7b3bba982a74118cf1c7b80f20349caeead244c584a94a8104b2f838bfb43e98217c526750ceb275bc113bb778f861bcc47d9bc90ad35a7104b28db012ee48f6decd30792658072639ffe3818896b6cf0f91a021da9ea49cc7c05c0f78924df764d9e7a12844de3436da5e2347fbd8f30a634111e353fb571ebf19b6ca7dc4870dc0d46ec677abac8c709bbc8ef06e3f0f3ecf637c01ca728618ed349b09afcc246122352b63c678e773e169a19b32c79e65d8cf56d29d65b81f54b5b8b74cb4b23b1dd9fbf03de57f26372aec94276174518a72c76eef22ee970e7c56ec9c9c49b8e3239cb55ce6bf958b703d6af75769d65dc043e189e83be91645ddff98d1b069eecfcd71d91e5ce3731cd796fbef2a76360247eaba73f1163386d5fa0fc4ecfaa3240b8aaec0717f2f73ae747c6eca34a60fd4192d5d7c2c59ed8309be28ccef1e64e5e6750ddcf4d76dec4dd83749fc68739df4165b2fd53e3bdbe2019f3b4b217942715f17c0626881f6d14e56ffddce7b4b413c971c9f9365f6a344f85a7f89f3471c628398703e7cf9a38c3949cbd80f3174d9c214ace5f61eefc55136794927307189dbf69e28c50726e059cbf6be2f453721e0e17fbbf34710629399f06ce3f34710628395702e79f36b68bddc7cdf9a6d5c93c7b9b6f6211a79e3057fedb0e5f2e45e46f05bebf24f93a9a2cf892644e315f386e0fb84cdd4d3b45ad97a38497ed21b0d1dbcea3aaf53223e795c0d9dea3aaf53223e75e10d9ece051d57a9991f334e05ccfa3aaf53223e7b5c0d9d1a3aaf53223e753c0d9c9a3aaf53223e76bc0e9f1a86abdccc8f927707a3daa5a2f33720e808552be24e74669c9d94d33908aa91f1aaf8167a1f52511b76e96a25d9637c68cc2b5276727367cddade20af3e6d5b23920dda4a0d7c068edcc02ed23847e0da0bb78d4b46df61166c3d74044a9ab16ca0025e59f40b981c74ac1d480fdd8a6d891c009c0d7cd63a9827722e7b8d0cc8a25e660aab5fbbdd8e1fd5ab8df6f2809e9fed5d569747c585d00abbb474d431d3fe1fcf7033cfbf4f0a869a8c348790e2c7036f2a869a8c348b90b506eec51d3508791f209a0ece951d3508791f256a0dcc4a3a6251e23e54740d94b92b2ab7937775558e9bca0c8ac1c13cfdba24b9fce52701d60e9d35b126e508b47f79b1ff4cbecec056375ed99304637f558387b91a29e972c4de69ee2ad3695e2e6247bbdb45e25aef96b2d558a5b2dd5cae470e0dfcc63a5444a3c874aa45c044fe67d3c6a5a99c4088ff9bf0fcf489bdbd81ff2998ccc44bdad2ae7e5a52695100f5db9afaa35af10e31c2dc55901b7b0be929c5bb632afae4b29481f6178e4489839b7f064b55b217efb966ac51648c53a7f8619b59f476de9523fe188fd03a684fed2f7fa4419809ae4987567d391261a9a387e5dc6b8e30e7c5b7a2c94d24fcda86506b3de84934b4da67b6e1508580177fe015a617d84b00b01762b49d85eeb54842a28d9fb456c7dfa115cfe03356132d6f4fa1830b7d6841922c4fc00300769c20c13627e0898db68c20c1062be009883356106190324803944132663e3dc4701735b4d987e42cce701733b4d988c71d03580b9bd26cc3821e6a780b98324e6364d1e2debcd3ed174937fec46980a933db32a9dcf56990a9db539f7b8391522a2433d2a7b0dfa299b91d7c173fc8e5a61199b91d703ac4f2b2c6333f27900ebd70acb982e311f60035a61199b911f0db041adb08ccdc8af06d8905658c666e4d7006c582b2c6339e095001bd10acb5887fe7a808d6a85656c467e17c0c63c2a3a6907089b925e010fb0718f9a1e9a71c6a2ff302277f258e8efba2e953f6384d5152f80bca89db581c609412f04d05db4818608414f04d05db5818609414f05d0ddb481460841cf02d0ddb5814609419702e81eda407d84a0c700e8306da08c45a98f05d0e1da400384a0c703e8086da04142d01300b440cf7e7f8c3249f717c01ca909933149f757c01ca509933149f76bc01cad09933149f727c01ca309933149f74bc01cab09933149f72bc01ca709933149f763c01caf09933149f70bc09ca009933149f737c02cd484c998a49b077b4513253187b650f9616eda19e522e7a5eea9d079ce1bb594b23bcabc7981f9c6679b6de3b936aa44886dd34d87fce849362a8baebdd856c4b4a412bc557514cb7282ade3c936ca162064e6850b4284650b6e84e9614f1bbbf281b514e2e2ae777f39dcb68a6c1485c1519a9823dd7138276f9cf36bb5993b5b0fe831c6efdbc37c3a4512b87fc6194ea3935f32c979938393635938ad7cb9d8197c980aa6da9e65336f1c10256c1c1001da69d9ac769fbea2650cf51d0393c05e362a432165cba374bcf3b1d0942f2c933e5ef6b614eb75c03add4ea9c2480e35483d1deefd7b7b14753b8e53a68fcd00ce7d3471c628398b80735f4d9c614ace2d81733f4d9c214ace6ec0b9bf26ce2825e74ec0798026ce0825e776c079a0264ecec4f0a5b0ce2cd6c419a4e4fc16380fd2c419a0e47c02384bb2fb9c8e679442849b1f5700e50c49ca011947938a92013acbed28bf93223e10a24aa53a0276cd7b38881e565c2576fc1b769cca6c35004eed9a8e75be649af361a94abe584de703600a289744ecf75f05948394bd683f81115aa1a7176d8cb181202cf167ea0ade311e6298069cb37405ef1839470067a5aee01d23e7a6c079b0aee01d23671e70ced615bc63e4dc1138ab7405ef1839070267b5aee01d23e72258d0cfd115bc63e45c0d9c35ba82778c9cd702e75c1d99214d0b6031561b7d16580fc966dd26fe04c6472060572b3d424b4da0c3cd531e69821e75a633d874334a67e54d32ff75234ce97dc4ea08e34c9bc008adb39167dfd850786da110be40487f8829d56b81f411421e0b17fa3c2d900142c8053022e7eb8972860929bf8567a10536dbaecf68e1a84c6e77afbd1c46eda192d4dd0d75e2af75d36717385f38d2e14b6cbda55ec697e4dd17eee80b6d8ed5d64e2665b6bc676c7ff119dcaa0ed3d3649db170c6a340f93749ca2eeece8af37fdce3b4891dcd5267d426be5c8cad488a2d0617fbe1926c7d9a3553772fea439c5fcbd73a1619ab3bf482b1f877bd9dbe238477f74d602175848d7ae0adad491beb818709eb81df0f97fd3ff4d40367ccb55b1f2ef645baca060628eb815f0f97f891da4019136b6f00d0a3b48132d6035f01a0476b0365ac077e35801ea30d94b11ef835007aac3650c67ae02b01f4386da08cdb719700e8626da08cf5c09703e8f1da4019531aae00d013b48132d603bf12404f9404ed9604f5db2f7221b68334079ed84fcae6137b7ab525c1e5fb1229ca383cb19f6c631cc69d8f32e793f5ce87a5426b8bda493596868bf8141bb56a82667727576ad5ac02be53f556530c128686c7c1657d9a8d8cced456467543f1a95cca47ea03fbe84b6c2c7f5abbe10c77beb4c60488dd9cba62d3ff284e9829f726809eae0d344608fa06809ea10dd44f087a3d809ea90dd447087a2d809ea50d3448087a13809ead0d3440087a03809ea30d344c087a0f80fe531b688810f436003d571b689410f455003d4f1b688410f425003d5f7769f4306169f48d8177a9ded2e821c23a363b42c8e4021bc13cb7468df5609e582cf42808e65d98cd987c8109dccd4866d109de9466b517a28cc2457e919e22fd8c4da74e819da28bf5ce9771c2f97206cc9797d838e5de366d2e9c728f02eb3249d6814d22f78d9b7299e00e73feeb8ee385b2117db1dbd5f7c07ca98d0d924c17fe31cad04921dcb22ed3c419a7e43c0a382fd7c4e9a7e4dc132ef6e59a3803949c15c0798526ce2025e745c079a526ce1025e7fdc0799526ce3025e78bc0b94213678492f31de0bc5a13679492b3372c94ae91e4dc00caa3276a33584a5e140b25bf01a1919536fa71a797b670ab7dd599c7cc4a137d6afafb5c2e75b10026816ba543cda5e68f4a1bc8c73694589befbcac8af094f12018b9d749e72cd7992cd00af3856e790b57c0bde867caf642a890c21b0d78d7ebaa31cf7816ee20d8f6b841578d7946cec9c079a3ae1af38c9cfd80f3265d35e61939bb00e7cdba6acc33724680f3165d35e619398700e72a5d35e61939cf8185d2adba6acc33727e099cb7e9aa31cfc8792f70deaeb3c6bc9ff0c9fc3160bd239b594ce9f36784f0cc5c5f08cddda985324849b90c28efd24219a0a49c0f94776ba10c5352be0894f768a10c5152de0f94f7dac93dcea57a20e3e0c9e73e1b452c6a4d84b6ca549b4dbcf71cda12e7ed60c973bf7c7fecb9695d5e8a4ce9e84ac7a42eafd0f96f9591996bdec0fdf351e6ed0accb73adbac2de7126e587407e2076c2cd6676654b1067338259b9b48e5c97f0697fd8392ac1ddd465ee622770bf2f3cd8fdbc2f87bc8461798cc9e6af820b7808ddb87b54032f625da0c201fd1021964bcb401f2512d908c0d9e0600e4635a2019db3b2d03c8c7b540860821cf07c827b4404609212f03c827b5403286292e05c8a7b440c609213f01c8a7b540c608213f00c867f4340d0b10c6810a81f2599bad420fcbb1679b532098fb9c8eb64c11c2b64caf41d0e27949c65eeb54054174af46ece21e09a1b2173461460931cb0173b526cc1021660c305fd484c9d8f8330e982f69c2645c100500f3654d984142cc2060bea20993b1a76a57c07c5513a69f10b33b60bea609334688590198af6bc2642c5e763860be21ddc63bd523a8c6d6c166b196366703db9b36d87c39c176173c6fbf658badd416db6c29b68f60b4bdada77b7490304d2a1f2290efd81881819cb8707f860bf75d49b6aece6f1a8b37b8c543dcb79b630ec5971156b0cc07b8f724e1b6ccb88265a1f9d666e58d30bcee9b959918ab68247c88589343b8a0dfd70eec2704f6c256c307da810384c07100fe503b709010b808803fb2712c34f18cb3c050d69a7c5e3cb768692d20767e713f588d7e6ce3f9db67b2cdcbcdf3770de496a75606a9d31282a572864a6196c0fa608d9e36b341c2356a0c56029f4852f64d52d63b3364a5f3d275a9c2ee235cc33e0763f453e9672617748af322f72f774f4314269b7df2d50d1c016c9f49b27992879e1215d8e6491e5fda5e2c591fb03eb771bc2e9a433da2e7c01df90b1b67bf526955f550a66e6c5e418b495691bc90ecbcd7516a4c5e02ebc62f25590735616d3c0796be6eccdd8e141540fd958d965ee9854253cf3b13cc49c5c4934e2ea50bae07bc5febe9f6c3b8953b0f6e54dfe8288a33def958689ed5135dd105b38ade166be309acdfeaeee917232cc4bc134c00dfe9ed51e5277c3aea0a8f9ddf4bd26ed64238a4f54a07168222629b74136132f8c146a992a041cd95368917c3e3c08f927c439aad5bab1a1650ffe9ea95b1d6fa8e30bbfe6463bc069c7bbcf5f12a76735a02e3f5e7ecae4ed9d31b2e8499f3171bcb28bc1955392f2f35f5fe27af752210bf31895de8b381f7573d0d687d840d68fbc0b2e9379b31ab4a331edd513acfe0b5be09e223dc04b90a6e45bfdbda9c2b36fd2a669b67a5d4ae679dd91c996be372178be00f851bd3bf6c04a5c3395430ef08982bffd0d9bb97b17a7018e6cd3f6d5ce0999d46f7531ebf3a1666cb7f6bc2643c7eb51830ffd284c978fc6a3c60e679156132c6ec8f06cc765ee1b213e921b942b3c55c6edec45d734e4cc63a45bba2f59782ec01f7f4f65e8ac6f121c200de81305e3b78d52d9d847741f7961abd7361e9b49e57515fd498e4969dd8d3d1c1304a3b6ae28c53729e0d9c9d3471fa2939a7c2c5eed1c419a0e43c0938bd9a3883949ccb81335f13678892f361e05c5f13679892f315e0ecac893342c9f9157076d1c419a5e4dc1c164a5dbd59dc2e4eef34c05890fc50d8e0d8c06b2571a138d9123eb1f96625ef5e6c246e0c7cddbc6a3af931f618e80017f5865e359dfc1829cf03caee5e359dfc18294703650faf9a4e7e8c94cf00e5465e359dfc1829ef04ca8db550462829bf04ca9e362813e932a95f273b7fec9ed92c808388c2c932629b16d36131b48924e5a62da41e0e33652adc9a0b23ccead24d412c37cf38bcd9471360b3ad979ed525630b87ade132efad6775c9487933506eaa6775c948792a506ea66775c948f91650f6d1b3ba64a47c082837f75ae9131db59ff62a969f79242c83faca8fc412d3ced8cd2f9a63d6909579f39b1c594d554912a3ec2d16e88505d0165e0b87561b0bfaece5688c70bee93213fa4dfd3adc3d10e17cd17ce755eef98b5ae9709b186d2718a5fdbc564e58c673e8c45a7bd8d1e92fc9d7c1f02d70461d5fbad52e70f96e2989d4b959cdd822f36b3de169720fa00df05a68ecd5da42a6b1b1978fb0b1d71570816e25c9d8c9f94d5df2d4a33b9ff11d191d06236ea0debb6c8cf02efb1edc65b7d691c7dbb49889e0c57dbed4d1d1e3e10965905745f3c320e11c590a17fe367636ff0339b4063c092eeec15ecb2575329f392384f3e62ab87b0ff1aaa856d45819ce3dd1e35602105d189d2d35778e02da6dbdaaeb6cf90827856781773be95b53e299a7698df6e623952f06b925dc9ab6f7aaa9b31123acb3b12b2c9676f05a6cb97d688ed5d03c0920877ad5d4d064ec147d1c5cde3b7a55d6d00c10d6d0bc19587d5e951537e2843b3a55704ff77bd594278b10c69a1e80111a90a474dedbe403cd4c2e8f6a244373627dffc60157d0ce2e4dc4fe13bad86de61378420fe99c0f4384f36129cc8761af702fc0b9ce1fd624e360ee3b0f4f5619147caa19295626142ee7888d7360335b7dae9996b61c8f4b2ec8f3a538a7c0e51dd5c419a3e4dc1938639a38c3949c1b02675c13678892f327983b77d2c419a5e4dc1e46e7ce9a3823949c038073174d9c7e4acebfc1c5beab26ce2025e753c0b99b26ce0025e732e0dc3d9b9b134d31f9286f8498fa1e5e0bad8f539b1335260ba97c2d7d29ac36efae90025e08c0c3b403fb088117433064b87430a40efaa24e3289feb5c9460b62705b89d5ce8059744476d267a6e415e48d77bee9d4e9dbd4af539d379b9d3c3e2a3826eb3a08d1f6838bbe404f819c28e1a6eff7b0b01f69b3e06d5bb91db9b5b7de0946ec283d479d1929bf831bd2683d479d1929cf80513946cf516746ca30508ed573d49991f231a01ca7e7a83323e52aa01cafa7900e23e5fb4039419272db260ba5aae4e3e4d8e40ab4c41c762e5fc76c63c65c631f90177a2db6a74fac4dff97dad3ef0fd413b33951a4e77e460993721e87f5e9241b49399905a3b16588e0082d90aabef53b3ca84eb67b8a33d7e7d9c32052b5a7de03b23ec2835e3f026d919d5a33b9547fb8075cf453ecf08572a854cf44e09b6aa3c75a66273a0294673a56c0a53c4d1326e322691560eea5093344887925604ed7841926c4bc0a30f7d684c9d87ef672c0dc47132663fbd9e580b9af264cc6f6b39702e67e9a3019dbcf5e0698fb6bc28c1162de06980768c28c1362de0d98076637aec97eb47553886b16db287fbf2e29e031c2dcd005f0747e9036d03821e822002dd1061a2204ad06d019da40c384a07301b4541b688410b40e40cbb481460941e70368b93650c6c334a5005aa10dd44f085a06a033b581329e519a09a0b3b4810609410f06d04af91a8975a61abc6be0e6d09739ff3e77bfb2d2f9922966173d9159cfb7f93b149edd0f96645cdf9461718fc3149aecba12d375ad9630dd600b209b2ddd91a0d88cbd69ce878be57eb36e3607df388b005a55366344e9994482cbf1255294bb438ca85a92b25fc6750ff7748c4a1a3a08559ad78ba669ed25965808e3748e8dc442ec3fe95eea534c2a568579b31293835cd8504e92b70be5c3c05a23c9da2dc91a733ecaa01faa850c23b19ea887c05267ae8dca0b382613f7ece669dae263512c61ab0cc6e2217a4bee8709974637c1ada956775df820610e7147384050a7a3a341e239c85d1d5498b7ab965d548975cc0ac2c8ad97a475ffea52536fa1da79c7e97963087bb2ed0633e83cbd332863b3a735300ee7ebe9591026ec59301028174852763194898ab225a6697c5543850531b6d1620702e1e23e54fa3840a9b9546bcd1ab3b1c9ec14b36c77df96ef08eb10e05ba8ab3c2f63fe46093cfb1ca6ab3c2f23e79ec0f9375de5791939fb03e7e1bacaf332726e009c7fd7559e9791330a9c47e82acfcbc8b92d70fe4357795e46cef360a1b44857795e46ceaf80f3485de57919391f02cea36c3c87fbd75212a598fa80f4be306f1e2ddf68d9dd441b61aac6969afa9b65ceb79c0b3be8611891c7e8692016263c657107501e6b6337320b1dabc522e50b00ef381d9b106e8153379a5e637e3727b9c32346bb4a8ab63b442c174bd26ed7622daec9ce17b895628bccaff579a3f326fd0f14e6d918d08fb7b1706a6dd9343cc95f6e36d4841b4089dda676803dca133471c629398f00ce133571fa293927c2c57e9226ce0025e73ec079b226ce2025e705c0798a26ce1025e7edc079aa26ce3025e70bc0799a26ce0825e79bc0b94413679492732358289d2ec9b9fd3a65c5bb31a785264baed2f9fccc8618542257bec43c37a5f2e5051f59bb48e526bf06c1bd336c1c46683b1a903b8711ca608a385333ae9f10f71f807b9666dc0021ee22c03d5b336e9010f748c03d47336e8810f706c0fda70edc9672ec452b8589ed1d7c088bb47335e3c608715f07dcf334e3060971c701eef99a710384b8230177a9665c3f21eeee807b81665c1f216e0c702fd48c1b25c47d19702fd28ccb78e0711ee05eac19374c88bb27e05ea2193744883b097097d9e846df366e7a773aec02e627ac373106802fd50eec2304ee0ec09769070e12025703f0e5da810384c06500bcdcc60d2e95a9586fba7f667e8b637ca83811429057e83a42cc580ffa40d8a2bc52d7116246cef1c07995ae23c48c9c7d817385ae23c48c9c5ee0bc5ad7116246ce10705ea3eb083123e736c0b952d7116246cec57014e95a5d478819395f07ceeb741d2166e4bc0938afcf6629afa6973a1fe575f0b879832465cf36cebea63f0df155a7da0b6e40376a818c11420e03c89bb440fa0921a7c22c79b316481f21e41480bc450b649010f21b805ca505324008390d206fd5021926845c0f6e36b769810c1142fe0023f2762d905142c83e3022efd002192184ec0d90774a3713aa33d551eaccd659a52995edbe3f5ffd9ee17019df95cd5248e98b6fc619f11b187f77db6884ee379bb9890ddb22331e13add3caed553aeb2a85b90c30ef91c4f435d9159fd76a15fcba75289a6429ad66750721fe8b204a74af8d69c1b7d6fe38cc35fb2e83917c9f8d6921f3a2487c47d3f7807c8efb3561c60931c700e6039a30198b267401cc073561fa09313700cc8734610608313704cc8735610609317b00e6239a30438c856600f3514d986142cc4d00f3314d981142ccde80f9b826cc2821e63680f9847c97ef1aa8a39b6a83258636542c2c0c81b927a57b8715272b35582fad2df6d47d263c753f95cd8c217c3bd1bbb4478af241b87c9f96a4f4baad6ecd5f3acf7941bd9b332bc735438aab002edc67a42fdcf46ae3134dadf1d49ce7f66be03bb1b33ef03d2bdd3d7696b9af9619c03ac2592e0e58cfd9c22a32a38caf1cd5ce80f5bc24567bb74baac3c5d78d7857207ac1c6b2d7675297cbcd6e60650bbd65dcd7d49a3f27ace80d98ab2531bb9afd68778baac27c618999fc0b9d5feb9c9b03df46ea28807bd1465b947ae762ad34fb7b9977c4163de621d6137b235816bf24df40aaf5edfed4bf5474495c2bc5782e8cd0976dcc93ad3790c2a78ba864ac45ecf9e2251893af68c28c1262be0c98af6ac2647cf25d0d98af69c20c1362be0898af6bc20c10623e0d986f68c20c12623e03986f6ac2f411623e09986f69c2f413623e05986f6bc28c1162be0198ef68c28c1362be0998ef66777f85bd856e67d85f794f9272d3ffa0920563c3f1cf616cbeaf0d344e08fa07807ea00d344408fa16807ea80d94b140c8bb00fa9136d00821e8fb00fab136d02821e81a005da30dd44708fa1c807ea20d94b132d0f300faa936d00021e80b00fa9936d02021e82b00fab9246827e737d526fb71b6d94614a3da5a8aaa27ec507e61a3009dcfeca3bbc90795ce0bb3741cb9b358620c8cbc2fb319ee9804c9cda2e18e2552943b41b8e3abec86e13072e4975c0489c58ede82cbfc6b4d987e42ccaf01f31b4d980142ccab60c6fc5613669010f36ec0fc4e13668810f32fc0fcdecec91a7f0e9dac590c7c3fe8590ac5089742236029f4a324e5a026456e5aabb49249bf1dd1f20c62a3760814b4f9c9464e318edaccb38a7d8439c5836014ff2c49bb790b37a7b647ac70115ab1b318f7c16af4976c56664b4f57604cf03a1a287f953e26957e248fef009e17b07e93c4eafb5f9cbe8813ce93df01ecef5a616384b09f03ecbfb4c20609614f05d83fb4c20608614f04d83fb5c2fa09616700ecbfb5c2322ef28b01f62fadb05142d84f00d6dd3955091b21847d0f60db6985653c43fc2ec0b6d70a1b22847d04603be45bc817c9bc84a89fb28ce8c710f45b4f1b286391db0f00b4a33650c6daac9f02682749d0fe2d80b61c3b1d9dfc92f4ed15bfe41a60b9581614f07af22d47fd5be69d6a36fedc386b8579bb6ad9d8805898707b88fa7b2569bb256923ce4799f3c97ae7c3523ad9a2764278f3600f353f5fb8a45e22203dddf9b5cc2ca5f876ec778085d1faf9e2556baa9c17943a9faaca9bec7cb84d3c1225a9d2774bf9f64043c0d8393f8b3bf70566993e23994522786399d55e88320237962e9294039becdca7fafdb4767b99d06a3f201f6117a0b1c0dc355fcd1e28e3e9f793e1e2df20df424278e66d68181bd11c0a23b39b26ce3825e70ae0dc5013a79f92733a2ccebb6be20c50722e01ce1e9a3883949c3703e7469a3843949ccf00e7c69a38c3949caf03674f4d9c114acebf8073134d9c514ace81b050eaa537a219238c681e0023b577be853ce6b6699b66de0709a34e938075d37c6bfd3052adb9dde7f9116603b98ab21f4627786edfcc66a4a931d69ec9d8c486c701c2887c7b9857fbe45b3914569c1cad6e60d9d2868658c9feebcc8686a76393371fd5eccd47987b705572ec5426c36d43f3469b7865b9912c757e3fc1647cd4e71de43c81efe7fc6fddcc0f70fed7d2e7db99dda49179879a66ddb5e6fdcacc7b573b7fea7e677579be09ce4b36778740b7e410186c3a2cb45edc60b09b5ed430f5b89fab705e39c3f97ce756a7a4c1194d574ddfa1c6fced35a601cb6053a5a2aea1d98ffb7db83f18f75f96b8b3243e37c00cb8f4cf1699462e95ce28aacb2b4c765baa745ee32ab97f3ecae814981fee6c738dcf75dea967b3772f347344e2a756926c40546b2a675465f4fa61c95130dff9dce0641f8b5223e80a4f329faf4deec10c6e569563b0b9324acde2c3cd011a697e4275e6673add78ce72dea3de34aea9357e8ddf499df3d589a06f89f9d7bb9f9b63bebbc47733a985267283db75375f9118f4eee5bac0f9fc48f37af77d53d7b87b21b433af4d7d65a1696152d230a74e35e3c3bd0a762c74d3d322ce7f12b94d431bb29a8626dbca57277f97fa7cdbefe98efcb5bf5b5f7778b76b7986abfa7fbb66dbfa178d6df86954247b96d499cfb6f52f6dfbdfd7faf5df652d3f951d27ba3f8b5cfef76fe1fedc933f76334c7b9909669af3932f4b5e9a939c1727a636b71357c7ff03542000ac26520800
```
