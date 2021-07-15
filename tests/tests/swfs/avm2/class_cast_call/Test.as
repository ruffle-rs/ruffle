package {
	public class Test {}
}

dynamic class ES4Class {
}

dynamic class ES4Subclass extends ES4Class {
	
}

trace("//ES4Class(null);");
trace(ES4Class(null));
trace("//ES4Class(undefined);");
trace(ES4Class(undefined));
trace("//ES4Class(new ES4Class());");
trace(ES4Class(new ES4Class()));
trace("//ES4Class(new ES4Subclass());");
trace(ES4Class(new ES4Subclass()));

trace("//ES4Subclass(null);");
trace(ES4Subclass(null));
trace("//ES4Subclass(undefined);");
trace(ES4Subclass(undefined));
trace("//ES4Subclass(new ES4Subclass());");
trace(ES4Subclass(new ES4Subclass()));