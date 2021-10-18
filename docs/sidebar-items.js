initSidebarItems({"enum":[["Offset","An offset from the current event."],["TimeUnit","A unit of time, used in LF."]],"fn":[["try_parse_duration","Parse a duration from a string. This is used for CLI parameter parsing in programs generated by LFC, specifically, to parse main parameters with `time` type, and scheduler options with time type."]],"macro":[["after","Shorthand for using After together with [delay]."],["assert_tag_is","Convenient macro to assert equality of the current tag. This is just shorthand for using `assert_eq!` with the syntax of [tag]."],["delay","Creates a [Duration] value using the same syntax as in LF."],["tag","Convenient macro to create a tag. This is just a shorthand for using the constructor together with the syntax of [delay]."]],"mod":[["prelude","The prelude that is imported at the top of reactor files generated by LFC."]],"struct":[["AssemblyCtx","Helper struct to assemble reactors during initialization. One assembly context is used per reactor, they can’t be shared."],["AssemblyError","An error occurring during initialization of the reactor program. Should never occur unless the graph is built by hand, and not by a Lingua Franca compiler."],["ComponentCreator",""],["Duration","A `Duration` type to represent a span of time, typically used for system timeouts."],["EventTag","The tag of an event."],["GlobalReactionId","Global identifier for a reaction."],["LocalReactionId","Type of a local reaction ID"],["LogicalAction","A logical action."],["MicroStep","Type of the microsteps of an EventTag."],["PhysicalAction","A physical action. Physical actions may only be used with the API of PhysicalSchedulerLink. See ReactionCtx::spawn_physical_thread."],["PhysicalActionRef","A reference to a physical action. This thing is cloneable and can be sent to async threads."],["PhysicalInstant","A measurement of a monotonically nondecreasing clock. Opaque and useful only with `Duration`."],["PhysicalSchedulerLink","A type that can affect the logical event queue to implement asynchronous physical actions. This is a “link” to the event system, from the outside world."],["Port","Represents a port, which carries values of type `T`. Ports reify the data inputs and outputs of a reactor."],["PortBank","Internal type, not communicated to reactions."],["ReactionCtx","The context in which a reaction executes. Its API allows mutating the event queue of the scheduler. Only the interactions declared at assembly time are allowed."],["ReactorId","The unique identifier of a reactor instance during execution."],["ReadablePort","A read-only reference to a port."],["ReadablePortBank","A read-only reference to a port bank."],["SchedulerOptions","Construction parameters for the scheduler."],["SyncScheduler","The runtime scheduler."],["Timer","A timer is conceptually a logical action that may re-schedule itself periodically."],["TriggerId","The ID of a trigger component."],["WritablePort","A write-only reference to a port."]],"trait":[["ReactionTrigger","Common trait for actions, ports, and timer objects handed to reaction functions. This is meant to be used through the API of ReactionCtx instead of directly."],["ReactorBehavior","The trait used by the framework to interact with the reactor during runtime."],["ReactorInitializer","Wrapper around the user struct for safe dispatch."]],"type":[["AssemblyResult",""]]});