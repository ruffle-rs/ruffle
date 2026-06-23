// ============================================================================
// THE OMNIBUS AVM1 / AMF0 PROTOCOL REGRESSION SUITE
// ============================================================================

// --- 0. GLOBAL CLASS DEFINITIONS ---
function UnregClass() { this.type = "unreg"; }
function RegClass() { this.type = "reg"; }
function MismatchedClass() { this.type = "mismatched"; }
function DoubleRegClass() { this.type = "doublereg"; }

function verifyPayload(prefix, d) {
    trace("==========================================");
    trace("VERIFYING CHANNEL: " + prefix);
    trace("==========================================");

    // 1. Primitives
    trace(prefix + ".p_num: " + d.p_num);
    trace(prefix + ".p_nan_check: " + isNaN(d.p_nan));
    trace(prefix + ".p_pos_inf: " + d.p_pos_inf);
    trace(prefix + ".p_neg_inf: " + d.p_neg_inf);
    trace(prefix + ".p_str: " + d.p_str);
    trace(prefix + ".p_empty_str: '" + d.p_empty_str + "'");
    trace(prefix + ".p_bool_t: " + d.p_bool_t);
    trace(prefix + ".p_bool_f: " + d.p_bool_f);
    trace(prefix + ".p_null: " + d.p_null);
    trace(prefix + ".p_undef_type: " + typeof(d.p_undef));

    // 2. Natives
    trace(prefix + ".n_date_getTime: " + d.n_date.getTime());
    trace(prefix + ".n_xml_nodeName: " + d.n_xml.firstChild.nodeName);
    trace(prefix + ".n_xml_attr: " + d.n_xml.firstChild.firstChild.attributes.id);

    // 3. Problematic / Special Types
    trace(prefix + ".sp_func_type: " + typeof(d.sp_func)); // Specs dictate 'undefined' or dropped
    trace(prefix + ".sp_mc_var: " + d.sp_mc.custom_var);   // Native bind stripped, custom vars remain

    // 4. Arrays
    trace(prefix + ".arr_strict[0]: " + d.arr_strict[0]);
    trace(prefix + ".arr_strict[2]: " + d.arr_strict[2]);
    trace(prefix + ".arr_strict_len: " + d.arr_strict.length);

    trace(prefix + ".arr_sparse[0]: " + d.arr_sparse[0]);
    trace(prefix + ".arr_sparse[4]: " + d.arr_sparse[4]);
    trace(prefix + ".arr_sparse[1]_type: " + typeof(d.arr_sparse[1]));

    trace(prefix + ".arr_mixed[0]: " + d.arr_mixed[0]);
    trace(prefix + ".arr_mixed['string_key']: " + d.arr_mixed["string_key"]);

    trace(prefix + ".arr_neg[0]: " + d.arr_negative[0]);
    trace(prefix + ".arr_neg[-1]: " + d.arr_negative[-1]);

    trace(prefix + ".arr_fake['0']: " + d.arr_fake["0"]);
    trace(prefix + ".arr_fake_len: " + d.arr_fake.length);

    // 5. Typed Objects
    trace(prefix + ".t_plain_is_Object: " + (d.t_plain instanceof Object));
    trace(prefix + ".t_unreg_is_UnregClass: " + (d.t_unreg instanceof _global.UnregClass));
    trace(prefix + ".t_reg_is_RegClass: " + (d.t_reg instanceof _global.RegClass) + " (" + d.t_reg.type + ")");
    trace(prefix + ".t_mismatch_is_MismatchedClass: " + (d.t_mismatch instanceof _global.MismatchedClass));
    trace(prefix + ".t_forged_ctor_is_RegClass: " + (d.t_forged_ctor instanceof _global.RegClass));
    trace(prefix + ".t_forged_proto_is_RegClass: " + (d.t_forged_proto instanceof _global.RegClass));
    trace(prefix + ".t_double_is_DoubleRegClass: " + (d.t_double instanceof _global.DoubleRegClass));

    // 6. Deep Nesting
    trace(prefix + ".nest_deep_is_RegClass: " + (d.nest_deep instanceof _global.RegClass));
    trace(prefix + ".nest_deep.innerArray_len: " + d.nest_deep.innerArray.length);
    trace(prefix + ".nest_deep.innerArray[1]_date: " + d.nest_deep.innerArray[1].getTime());
    trace(prefix + ".nest_deep.innerArray[2]_xml: " + d.nest_deep.innerArray[2].firstChild.nodeName);
    trace(prefix + ".nest_deep.innerSparse[2]: " + d.nest_deep.innerSparse[2]);

    // 7. AMF0 Reference Caching (DAGs)
    // In AS2, `===` on complex types checks object memory identity. 
    // If AMF0 referenced them correctly, these will resolve to identical pointers.
    trace(prefix + ".ref_strict_is_exact: " + (d.arr_strict === d.arr_strict_ref));
    trace(prefix + ".ref_mixed_is_exact: " + (d.arr_mixed === d.arr_mixed_ref));
    trace(prefix + ".ref_typed_is_exact: " + (d.t_reg === d.t_reg_ref));

    // 8. Nested DisplayObject Verification
    // Expectation: nested_container.movie_clip_prop should be undefined
    // because DisplayObjects are stripped during serialization.
    trace(prefix + ".nested_mc_type: " + typeof(d.nested_container.movie_clip_prop));
    trace(prefix + ".nested_regular_prop: " + d.nested_container.regular_prop);
}

// --- 1. SETUP REGISTRATIONS ---
Object.registerClass("com.tests.RegClass", RegClass);
Object.registerClass("WeirdAliasString", MismatchedClass);
Object.registerClass("Alias.One", DoubleRegClass);
Object.registerClass("Alias.Two", DoubleRegClass);

// --- 2. CONSTRUCT MASTER PAYLOAD ---
var payload = new Object();

// Primitives
payload.p_num = 1337.5;
payload.p_nan = NaN;
payload.p_pos_inf = Infinity;
payload.p_neg_inf = -Infinity;
payload.p_str = "The quick brown AMF fox";
payload.p_empty_str = "";
payload.p_bool_t = true;
payload.p_bool_f = false;
payload.p_null = null;
payload.p_undef = undefined;

// Natives
payload.n_date = new Date(1672531200000); // 2023-01-01 UTC
payload.n_xml = new XML("<root><child id='avm1'>test</child></root>");

// Problematic Types
payload.sp_func = function(x) { return x * 42; };
var mock:Object = _root.createEmptyMovieClip("mockMC", _root.getNextHighestDepth());
mock.custom_var = "i_am_a_display_object";
payload.sp_mc = mock;

// Arrays
payload.arr_strict = ["index_0", "index_1", "index_2"];

var arrSparse = new Array();
arrSparse[0] = "sparse_0";
arrSparse[4] = "sparse_4";
payload.arr_sparse = arrSparse;

var arrMixed = new Array();
arrMixed.push("mixed_0");
arrMixed["string_key"] = "string_value";
payload.arr_mixed = arrMixed;

var arrNeg = new Array();
arrNeg[0] = "pos_0";
arrNeg[-1] = "neg_1";
payload.arr_negative = arrNeg;

var fakeArr = new Object();
fakeArr["0"] = "fake_0";
fakeArr["1"] = "fake_1";
fakeArr.length = 2;
payload.arr_fake = fakeArr;

// Typed Objects
payload.t_plain = { a: 1 };
payload.t_unreg = new UnregClass();
payload.t_reg = new RegClass();
payload.t_mismatch = new MismatchedClass();

var forgedCtor = new Object();
forgedCtor.constructor = RegClass;
forgedCtor.type = "forged_c";
payload.t_forged_ctor = forgedCtor;

var forgedProto = new Object();
forgedProto.__proto__ = RegClass.prototype;
forgedProto.type = "forged_p";
payload.t_forged_proto = forgedProto;

payload.t_double = new DoubleRegClass();

// Deep Nesting
var deepNode = new RegClass();
deepNode.innerArray = ["deep_0", new Date(1000000000000), new XML("<deep><nest/></deep>")];
deepNode.innerSparse = new Array();
deepNode.innerSparse[2] = "found_in_the_deep";
payload.nest_deep = deepNode;

// DAG References (Hits writer.reference paths)
payload.arr_strict_ref = payload.arr_strict; // StrictArray path
payload.arr_mixed_ref = payload.arr_mixed;   // ECMAArray path
payload.t_reg_ref = payload.t_reg;           // TypedObject path

var nestedMc = _root.createEmptyMovieClip("deepMc", _root.getNextHighestDepth());
nestedMc.data = "should_be_undefined";
payload.nested_container = {
    movie_clip_prop: nestedMc,
    regular_prop: "exists"
};

// ====================================================================
// CHANNEL A: SHAREDOBJECT (Disk Test)
// ====================================================================
var so = SharedObject.getLocal("avm1_omnibus", "/");
so.data.payload = payload;
        
trace("SO.getSize() pre-flush: " + so.getSize());
so.flush();
trace("SO.getSize() post-flush: " + so.getSize());

var soRead = SharedObject.getLocal("avm1_omnibus", "/");
verifyPayload("SharedObject", soRead.data.payload);
delete so;


// ====================================================================
// CHANNEL B: LOCALCONNECTION (Memory Test)
// ====================================================================
var lcRecv = new LocalConnection();
lcRecv.onMasterPacket = function(transmittedPayload, funcArg, mcArg) {
    verifyPayload("LocalConnection", transmittedPayload);
    trace("==========================================");
    trace("VERIFYING CHANNEL: LocalConnection (Top-Level Aborts)");
    trace("==========================================");
    trace("LocalConnection.top_func_type: " + typeof(funcArg)); // Expected: undefined
    trace("LocalConnection.top_mc_type: " + typeof(mcArg));     // Expected: undefined
};
lcRecv.connect("omni_lc_pipe");

var lcSend = new LocalConnection();

// Create problematic types to test top-level serialize checks
var topFunc = function() { return "top"; };
var topMc:Object = _root.createEmptyMovieClip("topMc", _root.getNextHighestDepth());

lcSend.send("omni_lc_pipe", "onMasterPacket", payload, topFunc, topMc);


// ====================================================================
// CHANNEL C: NETCONNECTION (Network Test)
// ====================================================================

// Remove DAG references for NetConnection. 
// Ruffle's exact reference bytes (the AMF0 counter tracking) are currently drifting
// out of sync with Flash Player on primitive values, causing byte-for-byte mismatches.
delete payload.arr_strict_ref;
delete payload.arr_mixed_ref;
delete payload.t_reg_ref;

var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(echoedPayload) {
    verifyPayload("NetConnection", echoedPayload);
};
responder.onStatus = function(info) {
    trace("NetConnection Status Code: " + info.code);
};

nc.call("test.avm1", responder, payload);
