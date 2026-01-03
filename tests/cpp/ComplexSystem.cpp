// Complex system demonstrating various UML relationships

class Logger {
public:
    void log(std::string message) {}
};

class Auth {
public:
    bool authenticate() { return true; }
};

class Loggable {
public:
    virtual void logActivity() = 0;
};

class User {
private:
    std::string username;
    Logger* logger;  // Aggregation (pointer)

public:
    User(std::string username, Logger* logger) 
        : username(username), logger(logger) {}
    
    void login() {}
};

class Post {
private:
    std::string title;
    std::string content;
    User* author;  // Aggregation

public:
    Post(std::string title, User* author) 
        : title(title), author(author) {}
    
    std::string getTitle() { return title; }
};

class Admin : public User, public Auth, public Loggable {
private:
    Logger loggerInstance;  // Composition (direct member)

public:
    Admin(std::string username, Logger* inheritedLogger) 
        : User(username, inheritedLogger), loggerInstance() {}
    
    void deletePost(Post* post) {  // Dependency (method parameter)
        // Delete logic
    }
    
    void logActivity() override {
        // Log admin activity
    }
    
    Post* createPost(std::string title) {  // Dependency (return type)
        return nullptr;
    }
};
