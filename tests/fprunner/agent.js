console.log("Frida agent started")
const fridaPidfile = new File("flashplayer_pid", "w")
fridaPidfile.write(Process.id.toString())
fridaPidfile.close()

const logPathPtr = Memory.allocUtf8String("flash_log.txt");
var mmCfgPathPtr = null;

Interceptor.attach(Module.getExportByName(null, "open"), {
    onEnter(args) {
        const path = args[0].readUtf8String();
		if (path.endsWith("/.macromedia/Flash_Player/Logs/flashlog.txt")) {
			args[0] = logPathPtr
		} else if (mmCfgPathPtr && path.endsWith("mm.cfg")) {
			args[0] = mmCfgPathPtr
		}
    },
});

rpc.exports = {
	init: function(stage, parameters) {
		if (parameters['MM_CFG_PATH']) {
			console.log("Using mm.cfg path: " + parameters['MM_CFG_PATH'])
			mmCfgPathPtr = Memory.allocUtf8String(parameters['MM_CFG_PATH'])
		}
	}
}