from pathlib import Path

path = Path("packages/protocol/src/lib.rs")
text = path.read_text()

old_object = """    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        self.enter_nesting()?;
        let result = (|| {
            self.consume(b'{')?;
            self.skip_whitespace();
            let mut map = BTreeMap::new();
            if self.peek() == Some(b'}') {
                self.position += 1;
                return Ok(JsonValue::Object(map));
            }
            loop {
                self.skip_whitespace();
                let key = self.parse_string()?;
                self.skip_whitespace();
                self.consume(b':')?;
                let value = self.parse_value()?;
                map.insert(key, value);
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.position += 1;
                    }
                    Some(b'}') => {
                        self.position += 1;
                        break;
                    }
                    _ => return Err(self.error("expected , or } in object")),
                }
            }
            Ok(JsonValue::Object(map))
        })();
        self.depth -= 1;
        result
    }"""

new_object = """    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        self.enter_nesting()?;
        let result = self.parse_object_inner();
        self.depth -= 1;
        result
    }

    fn parse_object_inner(&mut self) -> Result<JsonValue, JsonError> {
        self.consume(b'{')?;
        self.skip_whitespace();
        let mut map = BTreeMap::new();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(b':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                }
                Some(b'}') => {
                    self.position += 1;
                    break;
                }
                _ => return Err(self.error("expected , or } in object")),
            }
        }
        Ok(JsonValue::Object(map))
    }"""

old_array = """    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        self.enter_nesting()?;
        let result = (|| {
            self.consume(b'[')?;
            self.skip_whitespace();
            let mut items = Vec::new();
            if self.peek() == Some(b']') {
                self.position += 1;
                return Ok(JsonValue::Array(items));
            }
            loop {
                let value = self.parse_value()?;
                items.push(value);
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.position += 1;
                    }
                    Some(b']') => {
                        self.position += 1;
                        break;
                    }
                    _ => return Err(self.error("expected , or ] in array")),
                }
            }
            Ok(JsonValue::Array(items))
        })();
        self.depth -= 1;
        result
    }"""

new_array = """    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        self.enter_nesting()?;
        let result = self.parse_array_inner();
        self.depth -= 1;
        result
    }

    fn parse_array_inner(&mut self) -> Result<JsonValue, JsonError> {
        self.consume(b'[')?;
        self.skip_whitespace();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                }
                Some(b']') => {
                    self.position += 1;
                    break;
                }
                _ => return Err(self.error("expected , or ] in array")),
            }
        }
        Ok(JsonValue::Array(items))
    }"""

for old, new in ((old_object, new_object), (old_array, new_array)):
    if text.count(old) != 1:
        raise SystemExit("parser closure target was not unique")
    text = text.replace(old, new)

path.write_text(text)
