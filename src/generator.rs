use crate::ast::*;
use serde_json::{
    Value, json, Map
};

impl Graph {
    fn to_json(&self) -> Value {
        let mut expressions = Map::new();

        expressions.insert("list".into(), self.items_to_json());
        if self.ticker.is_some() {
            expressions.insert("ticker".into(), json!(self.ticker.as_ref().unwrap().to_json()));
        }

        json!({
            "version": 11,
            "randomSeed": "8bc6db9edd77f61bb7f2b455cd3ef0c5",
            "graph": self.viewport.as_ref().unwrap().to_json(),
            "expressions": expressions,
            "includeFunctionParametersInRandomSeed": true,
            "doNotMigrateMovablePointStyle": true
        })
    }
    
    pub fn items_to_json(&self) -> Value {
        let mut id = 0;
        let mut out = Vec::new();

        for item in &self.items {
            out.extend(item.to_json(&mut id, None, None));
        }

        json!(out)
    }
}

impl Item {
    fn to_json(&self, id: &mut usize, parent_id: Option<usize>, parent_name: Option<&str>) -> Vec<Value> {
        match self {
            Item::Expression(expr) => expr.to_json(id, parent_id),
            Item::Folder(folder) => folder.to_json(id, parent_name),
            Item::Note(note) => note.to_json(id, parent_id),
        }
    }
}

impl Expression {
    fn to_json(
        &self,
        id: &mut usize,
        parent_id: Option<usize>,
    ) -> Vec<Value> {
        *id += 1;

        let mut obj = Map::new();

        obj.insert("type".into(), json!("expression"));
        obj.insert("id".into(), json!(id.to_string()));

        if let Some(pid) = parent_id {
            obj.insert("folderId".into(), json!(pid.to_string()));
        }

        match self {
            Self::Setting(setting) => {
                obj.insert("latex".into(), json!(self.to_latex(setting.expr.to_string())));

                obj.insert("color".into(), json!(setting.color));

                if let Some(val) = &setting.color_latex {
                    obj.insert("colorLatex".into(), json!(val.to_string()));
                }

                if let Some(val) = &setting.line_width {
                    obj.insert("lineWidth".into(), json!(val.to_string()));
                }

                if let Some(val) = &setting.line_opacity {
                    obj.insert("lineOpacity".into(), json!(val.to_string()));
                }

                if let Some(slider) = setting.slider.as_ref() {
                    let mut slider_map = Map::new();

                    if let Some(v) = &slider.min {
                        slider_map.insert("hardMin".into(), json!(true));
                        slider_map.insert("min".into(), json!(v.to_string()));
                    }
                    if let Some(v) = &slider.max {
                        slider_map.insert("hardMax".into(), json!(true));
                        slider_map.insert("max".into(), json!(v.to_string()));
                    }
                    if let Some(v) = &slider.step {
                        slider_map.insert("step".into(), json!(v.to_string()));
                    }

                    obj.insert("slider".into(), json!(slider_map));
                }

            
                if let Some(val) = &setting.point_size {
                    obj.insert("pointSize".into(), json!(val.to_string()));
                }    

                if let Some(val) = &setting.point_opacity {
                    obj.insert("pointOpacity".into(), json!(val.to_string()));
                }
            }
            _ => {
                obj.insert("latex".into(), json!(self.to_latex(self.to_string())));
            }
        }
        

        vec![Value::Object(obj)]
    }

    fn to_latex(&self, string: String) -> String {
        return string
            .replace("(", "\\left(")
            .replace(")", "\\right)")
            .replace("\\{", "\\left\\{")
            .replace("\\}", "\\right\\}")
            .replace("[", "\\left[")
            .replace("]", "\\right]")
            .replace("->", "\\to ")
            .replace("*", "\\cdot")
    }
}

impl Folder {
    fn to_json(
        &self,
        id: &mut usize,
        parent_title: Option<&str>
    ) -> Vec<Value> {
        *id += 1;
        let my_id = id.clone();

        let title = match parent_title {
            Some(parent) => format!("{} ➤ {}", parent, self.title),
            None => self.title.clone(),
        };

        let mut out = vec![json!({
            "type": "folder",
            "collapsed": "true",
            "id": my_id.to_string(),
            "title": title,
        })];

        for item in &self.items {
            out.extend(item.to_json(id, Some(my_id), Some(&title)));
        }

        out
    }
}

impl Note {
    fn to_json(
        &self,
        id: &mut usize,
        parent_id: Option<usize>,
    ) -> Vec<Value> {
        *id += 1;

        let mut obj = Map::new();

        obj.insert("type".into(), json!("text"));
        obj.insert("id".into(), json!(id.to_string()));
        obj.insert("text".into(), json!(self.text));

        if let Some(pid) = parent_id {
            obj.insert("folderId".into(), json!(pid.to_string()));
        }

        vec![Value::Object(obj)]
    }
}

impl Viewport {
    fn to_json(&self) -> Value {
        json!({
            "viewport": {
                "xmin": self.xmin.to_string(),
                "ymin": self.ymin.to_string(),
                "xmax": self.xmax.to_string(),
                "ymax": self.ymax.to_string(),
            },
            /* "yAxisScale": "logarithmic", */
            "xAxisArrowMode": "POSITIVE",
            "yAxisArrowMode": "POSITIVE",
            /* "xAxisLabel": "Hello", */
            "complex": self.complex,
            /* "squareAxes": false, */
            "__v12ViewportLatexStash": {
                "xmin": self.xmin.to_string(),
                "xmax": self.xmax.to_string(),
                "ymin": self.ymin.to_string(),
                "ymax": self.ymax.to_string(),
            }
        })
    }
}

impl Ticker {
    fn to_json(&self) -> Map<String, Value> {
        let mut ticker = Map::new();

        if let Some(expr) = self.step.as_ref() {
            ticker.insert("minStepLatex".into(), json!(expr.to_string()));
        }

        if let Some(expr) = self.run.as_ref() {
            ticker.insert("handlerLatex".into(), json!(expr.to_string()));
            ticker.insert("open".into(), json!(true));
        }

        ticker
    }
}

pub fn generate(graph: &Graph) -> serde_json::Value {
    graph.to_json()
}
