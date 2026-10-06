import {invoke} from '@tauri-apps/api/core'

//the machine's memory and what fuji's processes take, the numbers activity monitor and task manager show; memory.rs is the long version, and BucketMemory is the caller

export function memoryReport() { return invoke('memory_report') }//{installed, total, used, available, details: [{name, bytes}], processes: [{kind, pid, bytes}]}, every number in bytes. installed is what the machine was bought with and total what the system can use; details are the platform's own breakdown rows, named as its tool names them; processes is this process first, as host, then the web engine's by kind, each measured the way the platform's tool measures it. Off the mac and windows, the machine's rows and the host alone
