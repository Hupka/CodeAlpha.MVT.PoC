import AXSwift
import Cocoa

class GlobalAXState {
    var observerGlobalFocus: Observer?
    var appGlobalFocus: Application?

    init() {
        do {
            try updateState()
        } catch {
            NSLog("Error: Could not update Global State")
        }
        createTimer()
    }

    public func focusAppPID() -> Int32? {
        do {
            let pid = try appGlobalFocus?.pid()
            return pid
        } catch {
            NSLog("Error: Could not read PID of app: \(error)")
            return nil
        }
    }

    func createTimer() {
        _ = Timer.scheduledTimer(
            timeInterval: 0.1,
            target: self,
            selector: #selector(updateState),
            userInfo: nil,
            repeats: true
        )
    }

    @objc func updateState() throws {
        guard let focusedWindow = try systemWideElement.attribute(.focusedUIElement) as UIElement? else { return }
        guard let app = Application(forProcessID: try focusedWindow.pid()) else { return }

        // only continue when app has changed
        if appGlobalFocus == app {
            return
        }

        var updated = false
        observerGlobalFocus = app.createObserver { (_: Observer, element: UIElement, event: AXNotification, info: [String: AnyObject]?) in
            var elementDesc: String!
            if let role = try? element.role()!, role == .window {
                elementDesc = "\(element) \"\(try! (element.attribute(.title) as String?)!)\""
            } else {
                elementDesc = "\(element)"
            }
            print("\(event) on \(String(describing: elementDesc)); info: \(info ?? [:])")

            // Watch events on new windows
            if event == .mainWindowChanged {
                do {
                    try self.observerGlobalFocus!.addNotification(.uiElementDestroyed, forElement: element)
                    try self.observerGlobalFocus!.addNotification(.moved, forElement: element)
                } catch {
                    NSLog("Error: Could not watch [\(element)]: \(error)")
                }
            }

            // // Group simultaneous events together with --- lines
            if !updated {
                updated = true
                // Set this code to run after the current run loop, which is dispatching all notifications.
                DispatchQueue.main.async {
                    print("---")
                    updated = false
                }
            }
        }

        do {
            try observerGlobalFocus!.addNotification(.windowCreated, forElement: app)
            try observerGlobalFocus!.addNotification(.mainWindowChanged, forElement: app)
            try observerGlobalFocus!.addNotification(.moved, forElement: app)
            try observerGlobalFocus!.addNotification(.focusedWindowChanged, forElement: app)
        } catch {
            NSLog("Error: Could not add notifications: \(error)")
        }
    }
}
