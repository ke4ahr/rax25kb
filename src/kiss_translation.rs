// rax25kb - KISS Port Translation Module
// Part 2: KISS port number translation and frame routing

const KISS_FEND: u8 = 0xC0;
const KISS_FESC: u8 = 0xDB;
const KISS_TFEND: u8 = 0xDC;
#[allow(dead_code)]
const KISS_TFESC: u8 = 0xDD;

/// Extracts the KISS port number and command from a KISS frame
/// Returns (port_number, command, data_start_index)
fn extract_kiss_info(frame: &[u8]) -> Option<(u8, u8, usize)> {
    if frame.len() < 2 || frame[0] != KISS_FEND {
        return None;
    }
    
    // Find the command byte (first byte after FEND)
    let cmd_byte = frame[1];
    let port = (cmd_byte >> 4) & 0x0F;
    let command = cmd_byte & 0x0F;
    
    Some((port, command, 2))
}

/// Modifies the KISS port number in a frame
/// Takes a complete KISS frame and changes the port number
fn modify_kiss_port(frame: &[u8], new_port: u8) -> Vec<u8> {
    if frame.len() < 2 || frame[0] != KISS_FEND {
        return frame.to_vec();
    }
    
    let mut result = frame.to_vec();
    let cmd_byte = frame[1];
    let command = cmd_byte & 0x0F;
    
    // Construct new command byte with new port number
    result[1] = ((new_port & 0x0F) << 4) | command;
    
    result
}

/// Translates a KISS frame from one port to another
/// Handles both standard KISS and Extended KISS
pub struct KissPortTranslator {
    source_port: u8,
    dest_port: u8,
    source_extended: bool,
    dest_extended: bool,
}

impl KissPortTranslator {
    pub fn new(source_port: u8, dest_port: u8, source_extended: bool, dest_extended: bool) -> Self {
        KissPortTranslator {
            source_port,
            dest_port,
            source_extended,
            dest_extended,
        }
    }
    
    /// Translates a frame from source to destination
    /// If no translation needed (ports match and same KISS type), returns None
    /// Otherwise returns translated frame
    pub fn translate(&self, frame: &[u8]) -> Option<Vec<u8>> {
        // Extract current port info
        let (current_port, _command, _data_start) = extract_kiss_info(frame)?;
        
        // Check if this frame is for our source port
        if current_port != self.source_port {
            return None; // Not for us
        }
        
        // If source and dest are the same and same KISS type, no translation needed
        if self.source_port == self.dest_port && self.source_extended == self.dest_extended {
            return None;
        }
        
        // Translate the port number
        Some(modify_kiss_port(frame, self.dest_port))
    }
    
    /// Check if a frame should be routed through this translator
    pub fn should_route(&self, frame: &[u8]) -> bool {
        if let Some((port, _cmd, _idx)) = extract_kiss_info(frame) {
            port == self.source_port
        } else {
            false
        }
    }
}

/// Frame router that handles multiple cross-connects
pub struct FrameRouter {
    routes: Vec<Route>,
}

#[derive(Clone)]
struct Route {
    source_id: String,
    source_port: u8,
    dest_id: String,
    dest_port: u8,
    translator: Option<KissPortTranslator>,
}

impl FrameRouter {
    pub fn new() -> Self {
        FrameRouter {
            routes: Vec::new(),
        }
    }
    
    /// Add a route between two endpoints
    pub fn add_route(
        &mut self,
        source_id: String,
        source_port: u8,
        source_extended: bool,
        dest_id: String,
        dest_port: u8,
        dest_extended: bool,
    ) {
        let translator = if source_port != dest_port || source_extended != dest_extended {
            Some(KissPortTranslator::new(
                source_port,
                dest_port,
                source_extended,
                dest_extended,
            ))
        } else {
            None
        };
        
        let route = Route {
            source_id,
            source_port,
            dest_id,
            dest_port,
            translator,
        };
        
        self.routes.push(route);
    }
    
    /// Find destination for a frame coming from a specific source
    /// Returns (dest_id, dest_port, translated_frame)
    pub fn route_frame(&self, source_id: &str, frame: &[u8]) -> Option<(String, u8, Vec<u8>)> {
        // Extract port from frame
        let (frame_port, _cmd, _idx) = extract_kiss_info(frame)?;
        
        // Find matching route
        for route in &self.routes {
            if route.source_id == source_id && route.source_port == frame_port {
                // Found a route
                let translated = if let Some(ref translator) = route.translator {
                    translator.translate(frame).unwrap_or_else(|| frame.to_vec())
                } else {
                    frame.to_vec()
                };
                
                return Some((route.dest_id.clone(), route.dest_port, translated));
            }
        }
        
        None
    }
    
    /// Get all routes from a specific source
    pub fn get_routes_from(&self, source_id: &str) -> Vec<(u8, String, u8)> {
        self.routes
            .iter()
            .filter(|r| r.source_id == source_id)
            .map(|r| (r.source_port, r.dest_id.clone(), r.dest_port))
            .collect()
    }
}

/// KISS frame buffer that accumulates bytes until a complete frame is received
pub struct KissFrameBuffer {
    buffer: Vec<u8>,
    in_frame: bool,
}

impl KissFrameBuffer {
    pub fn new() -> Self {
        KissFrameBuffer {
            buffer: Vec::new(),
            in_frame: false,
        }
    }
    
    /// Add bytes to buffer and extract complete frames
    /// Returns vector of complete frames
    pub fn add_bytes(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        let mut frames = Vec::new();
        
        for &byte in data {
            if byte == KISS_FEND {
                if self.in_frame && !self.buffer.is_empty() {
                    // End of frame
                    self.buffer.push(byte);
                    frames.push(self.buffer.clone());
                    self.buffer.clear();
                    self.in_frame = false;
                } else {
                    // Start of frame
                    self.buffer.clear();
                    self.buffer.push(byte);
                    self.in_frame = true;
                }
            } else {
                if self.in_frame {
                    self.buffer.push(byte);
                }
            }
        }
        
        frames
    }
    
    /// Clear the buffer (e.g., on error or disconnect)
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.in_frame = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_kiss_info() {
        let frame = vec![KISS_FEND, 0x00, 0x01, 0x02, KISS_FEND];
        let (port, cmd, idx) = extract_kiss_info(&frame).unwrap();
        assert_eq!(port, 0);
        assert_eq!(cmd, 0);
        assert_eq!(idx, 2);
        
        let frame2 = vec![KISS_FEND, 0x15, 0x01, 0x02, KISS_FEND];
        let (port2, cmd2, _) = extract_kiss_info(&frame2).unwrap();
        assert_eq!(port2, 1);
        assert_eq!(cmd2, 5);
    }
    
    #[test]
    fn test_modify_kiss_port() {
        let frame = vec![KISS_FEND, 0x00, 0x01, 0x02, KISS_FEND];
        let modified = modify_kiss_port(&frame, 3);
        assert_eq!(modified[1], 0x30); // Port 3, Command 0
        
        let frame2 = vec![KISS_FEND, 0x15, 0x01, 0x02, KISS_FEND];
        let modified2 = modify_kiss_port(&frame2, 7);
        assert_eq!(modified2[1], 0x75); // Port 7, Command 5
    }
    
    #[test]
    fn test_translator() {
        let translator = KissPortTranslator::new(0, 1, false, false);
        let frame = vec![KISS_FEND, 0x00, 0x01, 0x02, KISS_FEND];
        let translated = translator.translate(&frame).unwrap();
        assert_eq!(translated[1], 0x10); // Port changed to 1
    }
    
    #[test]
    fn test_frame_buffer() {
        let mut buffer = KissFrameBuffer::new();
        
        // Add partial frame
        let data1 = vec![KISS_FEND, 0x00, 0x01];
        let frames1 = buffer.add_bytes(&data1);
        assert_eq!(frames1.len(), 0);
        
        // Complete the frame
        let data2 = vec![0x02, KISS_FEND];
        let frames2 = buffer.add_bytes(&data2);
        assert_eq!(frames2.len(), 1);
        assert_eq!(frames2[0], vec![KISS_FEND, 0x00, 0x01, 0x02, KISS_FEND]);
    }
    
    #[test]
    fn test_router() {
        let mut router = FrameRouter::new();
        router.add_route(
            "serial0".to_string(),
            0,
            false,
            "serial1".to_string(),
            1,
            true,
        );
        
        let frame = vec![KISS_FEND, 0x00, 0x01, 0x02, KISS_FEND];
        let result = router.route_frame("serial0", &frame);
        assert!(result.is_some());
        
        let (dest_id, dest_port, translated) = result.unwrap();
        assert_eq!(dest_id, "serial1");
        assert_eq!(dest_port, 1);
        assert_eq!(translated[1], 0x10); // Port changed to 1
    }
}

// Example usage:
//
// // Create a translator from standard KISS port 0 to XKISS port 2
// let translator = KissPortTranslator::new(0, 2, false, true);
//
// // Translate a frame
// let frame = vec![0xC0, 0x00, /* data */, 0xC0];
// if let Some(translated) = translator.translate(&frame) {
//     // translated frame now has port 2
// }
//
// // Use frame buffer to accumulate bytes
// let mut buffer = KissFrameBuffer::new();
// let frames = buffer.add_bytes(&incoming_data);
// for frame in frames {
//     // Process complete frame
// }
//
// // Use router for multiple cross-connects
// let mut router = FrameRouter::new();
// router.add_route("serial0", 0, false, "tcp", 0, false);
// router.add_route("serial0", 1, false, "serial1", 0, true);
//
// if let Some((dest, port, frame)) = router.route_frame("serial0", &incoming_frame) {
//     // Send frame to destination
// }