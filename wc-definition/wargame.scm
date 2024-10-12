
(struct TypeId
    (fields (u32 id)))

(struct Type
    (fields 
        (TypeId id)
        (string name)))

(struct Move
    (fields
        (string name)))

(struct Point3
    (fields 
        (i32 x)
        (i32 y)
        (i32 z)))

(struct Character
    (fields 
        (string name)
        (TypeId type-id)
        (Point3 position)
        (TypeId[] types)
        (Move[] moves)
        (i32 health)
        (i32 speed)
        (i32 attack)
        (i32 defense)
        (i32 sp-attack)
        (i32 sp-defense)
        (i32 level)
        (i32 exp)
        (i32 movement)
        ))