use crate::{rules::RuleForm, types::PrecedenceMaps};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeFuncInfo {
    pub title: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryInfo {
    pub title: String,
    pub active_pos: IndexMap<String, u8>,
    pub default_pos: IndexMap<String, u8>,
    pub active_cate: IndexMap<String, u8>,
    pub default_cate: IndexMap<String, u8>,
}

pub fn sample_rules() -> Vec<RuleForm> {
    serde_json::from_str(SAMPLE_RULES).expect("validated built-in rule presets")
}
pub fn sample_precedence_maps() -> IndexMap<String, PrecedenceMaps> {
    serde_json::from_str(SAMPLE_PRECEDENCE_MAPS).expect("validated built-in precedence maps")
}
pub fn sample_change_func_info() -> Vec<ChangeFuncInfo> {
    serde_json::from_str(CHANGE_INFO).expect("validated built-in change function information")
}
pub fn sample_dicts() -> Vec<DictionaryInfo> {
    serde_json::from_str(DICTIONARY_INFO).expect("validated built-in dictionary information")
}

// Preserved data from the original TypeScript package.
const SAMPLE_RULES: &str = r#"[
  {
    "id": "구엜룰",
    "metadata": {
      "title": "구엜룰",
      "updatedAt": 0,
      "color": "blue"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "신엜룰",
    "metadata": {
      "title": "신엜룰",
      "updatedAt": 0,
      "color": "indigo"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 1,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 0,
              "북한어": 0,
              "옛말": 0
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "넶룰",
    "metadata": {
      "title": "넶룰",
      "updatedAt": 0,
      "color": "violet"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 3,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 5,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "앞말잇기",
    "metadata": {
      "title": "앞말잇기",
      "updatedAt": 0,
      "color": "purple"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 1,
        "rawHeadIdx": 1,
        "tailDir": 0,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "노룰",
    "metadata": {
      "title": "노룰",
      "updatedAt": 0,
      "color": "fuchsia"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 0,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 2
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "반전룰",
    "metadata": {
      "title": "반전룰",
      "updatedAt": 0,
      "color": "pink"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 7,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "챈룰",
    "metadata": {
      "title": "챈룰",
      "updatedAt": 0,
      "color": "rose"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 8,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "듭2룰",
    "metadata": {
      "title": "듭2룰",
      "updatedAt": 0,
      "color": "red"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 9,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "천도룰",
    "metadata": {
      "title": "천도룰",
      "updatedAt": 0,
      "color": "orange"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 1,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 1,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 0,
              "북한어": 0,
              "옛말": 0
            }
          }
        },
        "regexFilter": "(.{3})",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 2
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "연결룰",
    "metadata": {
      "title": "연결룰",
      "updatedAt": 0,
      "color": "amber"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "붕어톱 궤휼 잎뽕",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "듭룰",
    "metadata": {
      "title": "듭룰",
      "updatedAt": 0,
      "color": "yellow"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 6,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "채린쿵따룰",
    "metadata": {
      "title": "채린쿵따룰",
      "updatedAt": 0,
      "color": "lime"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 0,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": "(.{3})",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 1
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "우샘룰",
    "metadata": {
      "title": "우샘룰",
      "updatedAt": 0,
      "color": "green"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 2,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 1,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "우샘어휘룰",
    "metadata": {
      "title": "우샘어휘룰",
      "updatedAt": 0,
      "color": "green"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 10,
            "pos": {
              "명사": 1,
              "의존명사": 1,
              "대명사": 1,
              "수사": 1,
              "부사": 1,
              "관형사": 1,
              "감탄사": 1,
              "구": 0,
              "무품사": 1
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 1
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "표샘룰",
    "metadata": {
      "title": "표샘룰",
      "updatedAt": 0,
      "color": "emerald"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 2,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 0,
              "북한어": 0,
              "옛말": 0
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "두샘룰",
    "metadata": {
      "title": "두샘룰",
      "updatedAt": 0,
      "color": "teal"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 2,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 0,
              "북한어": 0,
              "옛말": 0
            }
          }
        },
        "regexFilter": "(.{2})",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "옛두샘룰",
    "metadata": {
      "title": "옛두샘룰",
      "updatedAt": 0,
      "color": "cyan"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 2,
            "pos": {
              "명사": 1,
              "의존명사": 0,
              "대명사": 0,
              "수사": 0,
              "부사": 0,
              "관형사": 0,
              "감탄사": 0,
              "구": 0,
              "무품사": 0
            },
            "cate": {
              "일반어": 1,
              "방언": 0,
              "북한어": 0,
              "옛말": 1
            }
          }
        },
        "regexFilter": "(.{2})",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 1,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  },
  {
    "id": "로블록스 한국 끝말잇기",
    "metadata": {
      "title": "로블록스 한국 끝말잇기",
      "updatedAt": 0,
      "color": "sky"
    },
    "content": {
      "wordRule": {
        "words": {
          "type": "selected",
          "option": {
            "dict": 9,
            "pos": {
              "명사": 1,
              "의존명사": 1,
              "대명사": 1,
              "수사": 1,
              "부사": 1,
              "관형사": 1,
              "감탄사": 1,
              "구": 1,
              "무품사": 1
            },
            "cate": {
              "일반어": 1,
              "방언": 1,
              "북한어": 1,
              "옛말": 1
            }
          }
        },
        "regexFilter": ".*",
        "removedWords": "",
        "addedWords": ""
      },
      "wordConnectionRule": {
        "changeFuncIdx": 10,
        "headDir": 0,
        "rawHeadIdx": 1,
        "tailDir": 1,
        "rawTailIdx": 1
      },
      "postprocessing": {
        "removedWords": "",
        "manner": {
          "type": 0
        },
        "addedWords": ""
      }
    }
  }
]"#;

// Preserved data from the original TypeScript package.
const SAMPLE_PRECEDENCE_MAPS: &str = r#"{
  "구엜룰": {
    "edge": {
      "균": {
        "균": -1
      },
      "첩": {
        "첩": -1
      },
      "송": {
        "욱": -1
      }
    },
    "node": {}
  },
  "천도룰": {
    "edge": {},
    "node": {
      "틴": -10,
      "갠": -9,
      "걱": -9,
      "걷": -9,
      "겅": -9,
      "굇": -9,
      "굼": -9,
      "깐": -9,
      "깝": -9,
      "깩": -9,
      "깽": -9,
      "꺅": -9,
      "껀": -9,
      "껑": -9,
      "께": -9,
      "꽝": -9,
      "꽥": -9,
      "꿍": -9,
      "꿱": -9,
      "끔": -9,
      "끗": -9,
      "낡": -9,
      "냠": -9,
      "놈": -9,
      "놉": -9,
      "뇔": -9,
      "는": -9,
      "닿": -9,
      "돝": -9,
      "됨": -9,
      "둡": -9,
      "딘": -9,
      "뗀": -9,
      "뗑": -9,
      "똘": -9,
      "롭": -9,
      "른": -9,
      "맴": -9,
      "몫": -9,
      "묫": -9,
      "뭄": -9,
      "믈": -9,
      "밧": -9,
      "빡": -9,
      "빽": -9,
      "뺀": -9,
      "뻑": -9,
      "뻔": -9,
      "뻘": -9,
      "뻥": -9,
      "뼘": -9,
      "뽁": -9,
      "뽈": -9,
      "삵": -9,
      "솽": -9,
      "쉽": -9,
      "슷": -9,
      "쌕": -9,
      "쏨": -9,
      "씽": -9,
      "얀": -9,
      "옮": -9,
      "왹": -9,
      "잼": -9,
      "젉": -9,
      "즙": -9,
      "째": -9,
      "짹": -9,
      "쩍": -9,
      "쩔": -9,
      "쫀": -9,
      "쫌": -9,
      "찡": -9,
      "챗": -9,
      "챙": -9,
      "쾰": -9,
      "쿵": -9,
      "킁": -9,
      "킴": -9,
      "텁": -9,
      "톡": -9,
      "툭": -9,
      "팟": -9,
      "퍽": -9,
      "펴": -9,
      "푀": -9,
      "헉": -9,
      "홋": -9,
      "훑": -9,
      "흭": -9,
      "ㅁ": -8,
      "갹": -8,
      "겟": -8,
      "곶": -8,
      "괵": -8,
      "굉": -8,
      "긋": -8,
      "긱": -8,
      "껌": -8,
      "꽹": -8,
      "꿈": -8,
      "낼": -8,
      "넛": -8,
      "넴": -8,
      "늪": -8,
      "늬": -8,
      "댈": -8,
      "댐": -8,
      "던": -8,
      "뎀": -8,
      "뎅": -8,
      "딜": -8,
      "땃": -8,
      "땔": -8,
      "릎": -8,
      "맬": -8,
      "묻": -8,
      "밍": -8,
      "벋": -8,
      "벗": -8,
      "벳": -8,
      "뵐": -8,
      "뻗": -8,
      "뿜": -8,
      "삘": -8,
      "샴": -8,
      "섰": -8,
      "셧": -8,
      "솝": -8,
      "쇤": -8,
      "숄": -8,
      "숏": -8,
      "슴": -8,
      "싹": -8,
      "씌": -8,
      "얘": -8,
      "얽": -8,
      "없": -8,
      "웁": -8,
      "잿": -8,
      "좇": -8,
      "좨": -8,
      "죔": -8,
      "짙": -8,
      "짬": -8,
      "짱": -8,
      "첼": -8,
      "쳄": -8,
      "촐": -8,
      "칵": -8,
      "캅": -8,
      "캉": -8,
      "캥": -8,
      "콰": -8,
      "쾨": -8,
      "쿼": -8,
      "킨": -8,
      "탬": -8,
      "탭": -8,
      "톰": -8,
      "톳": -8,
      "툰": -8,
      "팃": -8,
      "팅": -8,
      "펭": -8,
      "퓰": -8,
      "햅": -8,
      "헵": -8,
      "홉": -8,
      "횔": -8,
      "휠": -8,
      "힉": -8,
      "겻": -7,
      "굵": -7,
      "깬": -7,
      "꽈": -7,
      "꾐": -7,
      "끕": -7,
      "끽": -7,
      "놓": -7,
      "닮": -7,
      "댑": -7,
      "덮": -7,
      "덴": -7,
      "둠": -7,
      "든": -7,
      "듬": -7,
      "떨": -7,
      "똑": -7,
      "렘": -7,
      "맵": -7,
      "멕": -7,
      "묏": -7,
      "뮈": -7,
      "밝": -7,
      "벚": -7,
      "볶": -7,
      "뺑": -7,
      "뽕": -7,
      "삐": -7,
      "얄": -7,
      "옭": -7,
      "옻": -7,
      "욋": -7,
      "줏": -7,
      "즈": -7,
      "쫄": -7,
      "쫑": -7,
      "쯔": -7,
      "찐": -7,
      "찻": -7,
      "챈": -7,
      "촛": -7,
      "캬": -7,
      "컵": -7,
      "퀼": -7,
      "탤": -7,
      "텍": -7,
      "튈": -7,
      "폼": -7,
      "폿": -7,
      "헹": -7,
      "홧": -7,
      "훌": -7,
      "훗": -7,
      "훙": -7,
      "휫": -7,
      "꼼": -6,
      "눅": -6,
      "닫": -6,
      "댕": -6,
      "덱": -6,
      "델": -6,
      "럭": -6,
      "믿": -6,
      "샹": -6,
      "싼": -6,
      "엮": -6,
      "윙": -6,
      "젠": -6,
      "줍": -6,
      "츠": -6,
      "컷": -6,
      "퍼": -6,
      "흩": -6,
      "꽉": -5,
      "땜": -5,
      "뚤": -5,
      "럼": -5,
      "렌": -5,
      "멈": -5,
      "씹": -5,
      "얹": -5,
      "젬": -5,
      "챌": -5,
      "챔": -5,
      "칩": -5,
      "턴": -5,
      "텀": -5,
      "펩": -5,
      "픽": -5,
      "껄": -4,
      "눌": -4,
      "덜": -4,
      "랙": -4,
      "룰": -4,
      "뻐": -4,
      "쉐": -4,
      "옌": -4,
      "욀": -4,
      "읍": -4,
      "쥘": -4,
      "켄": -4,
      "켈": -4,
      "퀘": -4,
      "펌": -4,
      "퓨": -4,
      "혹": -4,
      "횟": -4,
      "훤": -4,
      "휼": -4,
      "굿": -3,
      "댁": -3,
      "멤": -3,
      "잭": -3,
      "갸": -2,
      "겔": -2,
      "괄": -2,
      "꾀": -2,
      "꿸": -2,
      "넝": -2,
      "넬": -2,
      "닻": -2,
      "롄": -2,
      "멋": -2,
      "뫼": -2,
      "밸": -2,
      "샐": -2,
      "숯": -2,
      "쉰": -2,
      "싯": -2,
      "옳": -2,
      "킬": -2,
      "탯": -2,
      "템": -2
    }
  },
  "신엜룰": {
    "edge": {},
    "node": {
      "쇄": -1
    }
  },
  "채린쿵따룰": {
    "edge": {},
    "node": {
      "몫": -10,
      "츠": -10,
      "콕": -10,
      "갠": -9,
      "괵": -9,
      "굇": -9,
      "깩": -9,
      "깰": -9,
      "꺅": -9,
      "꽤": -9,
      "낼": -9,
      "덱": -9,
      "뎀": -9,
      "둡": -9,
      "뒛": -9,
      "딥": -9,
      "맬": -9,
      "묫": -9,
      "뭀": -9,
      "믓": -9,
      "믯": -9,
      "븍": -9,
      "븕": -9,
      "뼁": -9,
      "뽑": -9,
      "샬": -9,
      "셍": -9,
      "솽": -9,
      "숑": -9,
      "쉽": -9,
      "슷": -9,
      "싣": -9,
      "쐑": -9,
      "씁": -9,
      "씩": -9,
      "얀": -9,
      "얖": -9,
      "옫": -9,
      "왯": -9,
      "웁": -9,
      "잼": -9,
      "젉": -9,
      "젬": -9,
      "젼": -9,
      "짹": -9,
      "쨍": -9,
      "쫑": -9,
      "챗": -9,
      "쳄": -9,
      "쳠": -9,
      "츈": -9,
      "컷": -9,
      "쾨": -9,
      "킴": -9,
      "톰": -9,
      "툰": -9,
      "튱": -9,
      "팀": -9,
      "푀": -9,
      "헉": -9,
      "헨": -9,
      "흴": -9,
      "힉": -9,
      "갹": -8,
      "겡": -8,
      "됴": -8,
      "믠": -8,
      "볔": -8,
      "뵐": -8,
      "붝": -8,
      "빡": -8,
      "빤": -8,
      "뺌": -8,
      "뾩": -8,
      "삔": -8,
      "쇤": -8,
      "싄": -8,
      "옭": -8,
      "욋": -8,
      "윁": -8,
      "죔": -8,
      "죵": -8,
      "줏": -8,
      "쥰": -8,
      "즁": -8,
      "챔": -8,
      "첼": -8,
      "캅": -8,
      "탬": -8,
      "톧": -8,
      "튼": -8,
      "펌": -8,
      "헵": -8,
      "홉": -8,
      "횔": -8,
      "훋": -8,
      "휠": -8,
      "굉": -7,
      "긔": -7,
      "꼉": -7,
      "넢": -7,
      "벜": -7,
      "뿜": -7,
      "쌸": -7,
      "씌": -7,
      "졀": -7,
      "좇": -7,
      "짱": -7,
      "텟": -7,
      "끕": -6,
      "쌧": -6,
      "얄": -6
    }
  }
}"#;

// Preserved data from the original TypeScript package.
const CHANGE_INFO: &str = r#"[
  {
    "title": "없음"
  },
  {
    "title": "표준두음법칙"
  },
  {
    "title": "강제표준두음법칙"
  },
  {
    "title": "역표준두음법칙"
  },
  {
    "title": "강제역표준두음"
  },
  {
    "title": "자유두음법칙"
  },
  {
    "title": "양방향자유두음법칙"
  },
  {
    "title": "모음반전"
  },
  {
    "title": "자음반전"
  },
  {
    "title": "초성종성자유두음법칙"
  },
  {
    "title": "로블록스 한국 끝말잇기 두음법칙"
  }
]"#;

// Preserved data from the original TypeScript package.
const DICTIONARY_INFO: &str = r#"[
  {
    "title": "(구)표준국어대사전",
    "activePos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 1,
      "무품사": 1
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  },
  {
    "title": "(신)표준국어대사전",
    "activePos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 1,
      "무품사": 1
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    }
  },
  {
    "title": "우리말샘",
    "activePos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 1,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "activeCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  },
  {
    "title": "네이버 국어사전",
    "activePos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  },
  {
    "title": "끄투코리아 노인정",
    "activePos": {
      "명사": 0,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 1,
      "무품사": 1
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  },
  {
    "title": "끄투코리아 어인정",
    "activePos": {
      "명사": 0,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 1,
      "무품사": 1
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  },
  {
    "title": "끄투3 기초 사전",
    "activePos": {
      "명사": 0,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 0,
      "감탄사": 1,
      "구": 1,
      "무품사": 0
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    }
  },
  {
    "title": "끄투3 표준 사전",
    "activePos": {
      "명사": 0,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 0,
      "감탄사": 1,
      "구": 1,
      "무품사": 0
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    }
  },
  {
    "title": "끄투3 복합 사전",
    "activePos": {
      "명사": 0,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 0,
      "감탄사": 1,
      "구": 1,
      "무품사": 0
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  },
  {
    "title": "로블록스 한국 끝말잇기 사전",
    "activePos": {
      "명사": 0,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 1,
      "무품사": 1
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  },
  {
    "title": "우리말샘(어휘)",
    "activePos": {
      "명사": 0,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 0,
      "무품사": 1
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  },
  {
    "title": "우리말샘(어휘 + 구)",
    "activePos": {
      "명사": 0,
      "의존명사": 0,
      "대명사": 0,
      "수사": 0,
      "부사": 0,
      "관형사": 0,
      "감탄사": 0,
      "구": 0,
      "무품사": 0
    },
    "defaultPos": {
      "명사": 1,
      "의존명사": 1,
      "대명사": 1,
      "수사": 1,
      "부사": 1,
      "관형사": 1,
      "감탄사": 1,
      "구": 1,
      "무품사": 1
    },
    "activeCate": {
      "일반어": 0,
      "방언": 0,
      "북한어": 0,
      "옛말": 0
    },
    "defaultCate": {
      "일반어": 1,
      "방언": 1,
      "북한어": 1,
      "옛말": 1
    }
  }
]"#;
