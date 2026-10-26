# Dump native stacks of every thread in a process: python nstack.py <pid> [filter]
import ctypes, os, sys
from ctypes import wintypes as w

pid = int(sys.argv[1])
k32 = ctypes.WinDLL("kernel32", use_last_error=True)
dbg = ctypes.WinDLL(r"C:\Windows\System32\dbghelp.dll")

class THREADENTRY32(ctypes.Structure):
    _fields_ = [("dwSize", w.DWORD), ("cntUsage", w.DWORD), ("th32ThreadID", w.DWORD),
                ("th32OwnerProcessID", w.DWORD), ("tpBasePri", w.LONG), ("tpDeltaPri", w.LONG), ("dwFlags", w.DWORD)]

class ADDRESS64(ctypes.Structure):
    _fields_ = [("Offset", ctypes.c_uint64), ("Segment", w.WORD), ("Mode", w.DWORD)]

class STACKFRAME64(ctypes.Structure):
    _fields_ = [("AddrPC", ADDRESS64), ("AddrReturn", ADDRESS64), ("AddrFrame", ADDRESS64), ("AddrStack", ADDRESS64),
                ("AddrBStore", ADDRESS64), ("FuncTableEntry", ctypes.c_void_p), ("Params", ctypes.c_uint64 * 4),
                ("Far", w.BOOL), ("Virtual", w.BOOL), ("Reserved", ctypes.c_uint64 * 3), ("KdHelp", ctypes.c_byte * 160)]

class SYMBOL_INFOW(ctypes.Structure):
    _fields_ = [("SizeOfStruct", w.ULONG), ("TypeIndex", w.ULONG), ("Reserved", ctypes.c_uint64 * 2),
                ("Index", w.ULONG), ("Size", w.ULONG), ("ModBase", ctypes.c_uint64), ("Flags", w.ULONG),
                ("Value", ctypes.c_uint64), ("Address", ctypes.c_uint64), ("Register", w.ULONG),
                ("Scope", w.ULONG), ("Tag", w.ULONG), ("NameLen", w.ULONG), ("MaxNameLen", w.ULONG),
                ("Name", w.WCHAR * 512)]

CONTEXT_SIZE = 1232
k32.OpenProcess.restype = w.HANDLE
k32.OpenThread.restype = w.HANDLE
hproc = k32.OpenProcess(0x1F0FFF, False, pid)
dbg.SymSetOptions(0x2 | 0x10 | 0x4)
dbg.SymInitializeW.argtypes = [w.HANDLE, w.LPCWSTR, w.BOOL]
dbg.SymInitializeW(hproc, os.environ.get("SYMBOL_PATH"), True)  # folders with pumpkin.pdb / patchbukkit.pdb, ';'-separated
dbg.SymFunctionTableAccess64.restype = ctypes.c_void_p
dbg.SymFunctionTableAccess64.argtypes = [w.HANDLE, ctypes.c_uint64]
dbg.SymGetModuleBase64.restype = ctypes.c_uint64
dbg.SymGetModuleBase64.argtypes = [w.HANDLE, ctypes.c_uint64]
FTA = ctypes.WINFUNCTYPE(ctypes.c_void_p, w.HANDLE, ctypes.c_uint64)(lambda h, a: dbg.SymFunctionTableAccess64(h, a))
GMB = ctypes.WINFUNCTYPE(ctypes.c_uint64, w.HANDLE, ctypes.c_uint64)(lambda h, a: dbg.SymGetModuleBase64(h, a))

def sym(addr):
    s = SYMBOL_INFOW(); s.SizeOfStruct = 88; s.MaxNameLen = 512
    d = ctypes.c_uint64()
    if dbg.SymFromAddrW(hproc, ctypes.c_uint64(addr), ctypes.byref(d), ctypes.byref(s)):
        return s.Name
    return f"{addr:#x}"

snap = k32.CreateToolhelp32Snapshot(0x4, 0)
te = THREADENTRY32(); te.dwSize = ctypes.sizeof(te)
tids = []
ok = k32.Thread32First(snap, ctypes.byref(te))
while ok:
    if te.th32OwnerProcessID == pid:
        tids.append(te.th32ThreadID)
    ok = k32.Thread32Next(snap, ctypes.byref(te))

flt = sys.argv[2] if len(sys.argv) > 2 else None
k32.GetThreadDescription.argtypes = [w.HANDLE, ctypes.POINTER(w.LPWSTR)]
for tid in tids:
    ht = k32.OpenThread(0x1FFFFF, False, tid)
    name = w.LPWSTR()
    k32.GetThreadDescription(ht, ctypes.byref(name))
    k32.SuspendThread(ht)
    buf = (ctypes.c_byte * (CONTEXT_SIZE + 16))()
    off = (16 - ctypes.addressof(buf) % 16) % 16
    ctx = ctypes.addressof(buf) + off
    ctypes.c_uint32.from_address(ctx + 0x30).value = 0x10000B  # CONTEXT_FULL (AMD64)
    frames = []
    if k32.GetThreadContext(ht, ctypes.c_void_p(ctx)):
        sf = STACKFRAME64()
        sf.AddrPC.Offset = ctypes.c_uint64.from_address(ctx + 0xF8).value; sf.AddrPC.Mode = 3
        sf.AddrFrame.Offset = ctypes.c_uint64.from_address(ctx + 0xA0).value; sf.AddrFrame.Mode = 3
        sf.AddrStack.Offset = ctypes.c_uint64.from_address(ctx + 0x98).value; sf.AddrStack.Mode = 3
        for _ in range(60):
            if not dbg.StackWalk64(0x8664, hproc, ht, ctypes.byref(sf), ctypes.c_void_p(ctx), None, FTA, GMB, None):
                break
            if sf.AddrPC.Offset == 0:
                break
            frames.append(sym(sf.AddrPC.Offset))
    k32.ResumeThread(ht)
    text = "\n".join("   " + f[:200] for f in frames)
    if flt is None or flt in text or (name.value and flt in name.value):
        print(f"== tid {tid} name={name.value!r}\n{text}")
