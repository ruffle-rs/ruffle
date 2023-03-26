package {
	import flash.display.MovieClip;

	public class Test {
		public function Test(main: MovieClip) {
			doChildDomainLoad(main);
		}
	}
}
import flash.display.MovieClip;
import flash.display.Loader;
import flash.utils.getDefinitionByName;
import flash.net.URLRequest;
import flash.events.Event;
import flash.system.LoaderContext;
import flash.system.ApplicationDomain;

function doChildDomainLoad(main: MovieClip) {
	trace("\nStarting child domain load")
	try {
		getDefinitionByName("DuplicateClass")
	} catch (e) {
		trace("getDefinitionByName(\"DuplicateClass\") not found initially: " + e);
	}

	var loader = new Loader();
	loader.contentLoaderInfo.addEventListener(Event.INIT, function (e) {
		trace("Init event: DuplicateClass=" + loader.contentLoaderInfo.applicationDomain.getDefinition("DuplicateClass").NAME);
		try {
			getDefinitionByName("DuplicateClass")
		} catch (e) {
			trace("getDefinitionByName(\"DuplicateClass\") not found afterwards: " + e);
		}
		doOtherChildDomainLoad(main);
	});
	loader.load(new URLRequest("loader_domain_child/loader_domain_child.swf"));

	main.addChild(loader);
}

function doSameDomainLoad(main: MovieClip, otherChildDomain: ApplicationDomain) {
	trace("\nStarting same domain load");
	try {
		getDefinitionByName("DuplicateClass")
	} catch (e) {
		trace("same domain: DuplicateClass not found initially: " + e);
	}

	var loader = new Loader();
	loader.contentLoaderInfo.addEventListener(Event.INIT, function (e) {
		trace("Init event: DuplicateClass from domain gives: " + loader.contentLoaderInfo.applicationDomain.getDefinition("DuplicateClass").NAME);
		trace("getDefinitionByName(\"DuplicateClass\") gives: " + getDefinitionByName("DuplicateClass").NAME);

		var duplicateClassObj = otherChildDomain.getDefinition("DuplicateClass");
		trace("Already loaded class: " + duplicateClassObj.NAME);

		var clip = new duplicateClassObj();
		trace("Instantiate clip: " + clip);

		doChildDomainLoadAgain(main);
	});
	loader.load(new URLRequest("loader_same_domain/loader_same_domain.swf"), new LoaderContext(false, ApplicationDomain.currentDomain));

	main.addChild(loader);
}


function doOtherChildDomainLoad(main: MovieClip) {
	trace("\nStarting other child domain load")
	try {
		getDefinitionByName("DuplicateClass")
	} catch (e) {
		trace("getDefinitionByName(\"DuplicateClass\") not found initially: " + e);
	}

	var loader = new Loader();
	loader.contentLoaderInfo.addEventListener(Event.INIT, function (e) {
		trace("Init event: DuplicateClass=" + loader.contentLoaderInfo.applicationDomain.getDefinition("DuplicateClass").NAME);
		try {
			getDefinitionByName("DuplicateClass")
		} catch (e) {
			trace("getDefinitionByName(\"DuplicateClass\") not found afterwards: " + e);
		}
		doSameDomainLoad(main, loader.contentLoaderInfo.applicationDomain);
	});
	loader.load(new URLRequest("loader_domain_other_child/loader_domain_other_child.swf"));

	main.addChild(loader);
}

function doChildDomainLoadAgain(main: MovieClip) {
	trace("\nStarting repeated child domain load")
	try {
		getDefinitionByName("DuplicateClass")
	} catch (e) {
		trace("getDefinitionByName(\"DuplicateClass\") not found initially: " + e);
	}

	var loader = new Loader();
	loader.contentLoaderInfo.addEventListener(Event.INIT, function (e) {
		var duplicateClassObj = loader.contentLoaderInfo.applicationDomain.getDefinition("DuplicateClass");
		trace("Init event: DuplicateClass=" + duplicateClassObj.NAME);
		try {
			getDefinitionByName("DuplicateClass")
		} catch (e) {
			trace("getDefinitionByName(\"DuplicateClass\") not found afterwards: " + e);
		}

		var clip = new duplicateClassObj();
		trace("Instantiate clip in doChildDomainLoadAgain: " + clip);
	});
	loader.load(new URLRequest("loader_domain_child/loader_domain_child.swf"));

	main.addChild(loader);
}