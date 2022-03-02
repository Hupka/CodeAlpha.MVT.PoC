import Cocoa
import Vapor

let consoleIO = ConsoleIO()

public func configure(_ app: Application) throws {
	app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

	// Register AppDelegate to interact with AX features
	let appDelegate = AXAppDelegate()
	let application = NSApplication.shared
	application.setActivationPolicy(NSApplication.ActivationPolicy.accessory)
	application.delegate = appDelegate

	// Spin up websocket manager; manages all connected clients
	let websocketManager = WebsocketManager(eventLoop: app.eventLoopGroup.next())

	// Start state listeners
	let xCodeAXState = XCodeAXState(websocketManager)
	let globalAXState = GlobalAXState(websocketManager)

	// Setup Websocket routes
	app.webSocket("channel") { _, ws in

		ws.onBinary { _, buffer in
			if let msg = buffer.decodeWebsocketMessage(Connect.self) {
				let wsClient = WebsocketClient(id: msg.client, socket: ws)
				websocketManager.connect(client: wsClient)
			}

			if let msg = buffer.decodeWebsocketMessage(Request.self) {
				switch msg.data.type {
				case .getXCodeEditorContent:
					xCodeAXState.notifyEditorContent()
				case .getXCodeFocusStatus:
					xCodeAXState.notifyXCodeFocusStatus()
				case .getAppFocusState:
					globalAXState.notifyAppFocusStatus()
				default:
					break
				}
			}

			if let msg = buffer.decodeWebsocketMessage(XCodeEditorContent.self) {
				xCodeAXState.updateEditorContent(msg.data.content)
			}

			ws.onText { ws, _ in
				ws.send("pong")
			}
		}
	}

	// Register XPC listener to receive requests from clients
	// let machServiceName = "com.codeAlpha.AXServerXPC"
	// let delegate = ServiceDelegate()
	// let listener = NSXPCListener(machServiceName: machServiceName)
	// listener.delegate = delegate
	// listener.resume()
}
